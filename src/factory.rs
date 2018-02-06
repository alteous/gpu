//! Factory.

#![allow(dead_code)]

use buffer;
use gl;
use image;
use program;
use shader;
use std::{ffi, mem, ptr};
use texture;
use util;
use vertex_array;

use draw_call::{DrawCall, Kind};
use framebuffer::{
    ColorAttachment,
    ClearOp,
    ClearColor,
    ClearDepth,
    Framebuffer,
    MAX_COLOR_ATTACHMENTS,
};
use program::{
    Invocation,
    UniformBlockBinding,
    SamplerBinding,
    MAX_UNIFORM_BLOCKS,
    MAX_SAMPLERS,
};
use pipeline::{PolygonMode, State, Viewport};
use queue::Queue;
use renderbuffer::Renderbuffer;
use {Buffer, Program, Texture2, VertexArray};

/// OpenGL memory manager.
#[derive(Clone)]
pub struct Factory {
    /// Function pointers to the OpenGL backend.
    backend: gl::Backend,

    /// Destroyed buffers arrive here to be destroyed or recycled.
    buffer_queue: Queue<buffer::Id>,

    /// Destroyed textures arrive here to be destroyed or recycled.
    texture_queue: Queue<texture::Id>,
    
    /// Destroyed vertex arrays arrive here to be destroyed or recycled.
    vertex_array_queue: Queue<vertex_array::Id>,

    /// Destroyed GLSL programs arrive here to be destroyed or recycled.
    program_queue: Queue<program::Destroyed>,
}

impl Factory {
    /// Constructor.
    pub fn new<F>(query_proc_address: F) -> Self
        where F: FnMut(&str) -> *const ()
    {
        Self {
            backend: gl::Backend::load(query_proc_address),
            buffer_queue: Queue::new(),
            texture_queue: Queue::new(),
            vertex_array_queue: Queue::new(),
            program_queue: Queue::new(),
        }
    }

    /// Clear the color buffer.
    pub fn clear(&self, framebuffer: &Framebuffer, op: ClearOp) {
        self.backend.bind_framebuffer(framebuffer.id());
        let mut ops = 0;
        match op.color {
            ClearColor::Yes { r, g, b, a } => {
                self.backend.clear_color(r, g, b, a);
                ops |= gl::COLOR_BUFFER_BIT;
            }
            ClearColor::No => {}
        }
        match op.depth {
            ClearDepth::Yes { z } => {
                self.backend.clear_depth(z);
                ops |= gl::DEPTH_BUFFER_BIT;
            }
            ClearDepth::No => {}
        }
        self.backend.clear(ops);
    }

    /// (Re)-initialize the contents of a [`Buffer`].
    ///
    /// [`Buffer`]: buffer/struct.Buffer.html
    pub fn initialize_buffer<T>(&self, buffer: &Buffer, data: &[T]) {
        self.backend.bind_buffer(buffer.id(), buffer.kind().as_gl_enum());
        self.backend.buffer_data(
            buffer.kind().as_gl_enum(),
            data.len() * mem::size_of::<T>(),
            data.as_ptr() as *const _,
            buffer.usage().as_gl_enum(),
        );
        self.backend.bind_buffer(0, buffer.kind().as_gl_enum());
    }

    /// Overwrite part of a buffer.
    pub fn overwrite_buffer<T>(&self, slice: buffer::Slice, data: &[T]) {
        self.backend.bind_buffer(slice.id(), slice.kind().as_gl_enum());
        self.backend.buffer_sub_data(slice.kind().as_gl_enum(), slice.offset(), slice.length(), data.as_ptr());
        self.backend.bind_buffer(0, slice.kind().as_gl_enum());
    }

    /// Create an uninitialized GPU buffer.
    pub fn buffer(&self, kind: buffer::Kind, usage: buffer::Usage) -> Buffer {
        let id = self.backend.gen_buffer();
        let size = 0;
        let tx = self.buffer_queue.tx();
        Buffer::new(id, kind, size, usage, tx)
    }

    /// A collection of GPU buffers that may be drawn with a material.
    pub fn vertex_array(
        &self,
        attributes: [Option<vertex_array::Attribute>; vertex_array::MAX_ATTRIBUTES],
        indices: Option<vertex_array::Indices>,
    ) -> VertexArray {
        let id = self.backend.gen_vertex_array();
        let tx = self.vertex_array_queue.tx();

        // Setup the vertex array
        {
            self.backend.bind_vertex_array(id);
            if let Some(ref accessor) = indices {
                self.backend.bind_buffer(accessor.buffer().id(), gl::ELEMENT_ARRAY_BUFFER);
            }
            for binding in 0 .. vertex_array::MAX_ATTRIBUTES {
                if let Some(ref accessor) = attributes[binding] {
                    self.backend.bind_buffer(accessor.buffer().id(), gl::ARRAY_BUFFER);
                    self.backend.enable_vertex_attrib_array(binding as _);
                    self.backend.vertex_attrib_pointer(
                        binding as u8,
                        accessor.format().size() as _,
                        accessor.format().gl_data_type(),
                        accessor.format().norm(),
                        accessor.stride() as _,
                        accessor.offset(),
                    )
                }
            }
            self.backend.bind_vertex_array(0);
        }

        VertexArray::new(id, attributes, indices, tx)
    }

    /// Compile GLSL shader code into a shader object.
    pub fn shader(
        &self,
        kind: shader::Kind,
        sources: &shader::Source,
    ) -> shader::Object {
        let id = self.backend.create_shader(kind.as_gl_enum());
        self.backend.shader_source(id, sources);
        self.backend.compile_shader(id);
        let tx = self.program_queue.tx();
        shader::Object::new(id, kind, tx)
    }

    /// Link GLSL objects to create a GLSL program.
    pub fn program(
        &self,
        vertex: &shader::Object,
        fragment: &shader::Object,
        bindings: &program::Bindings,
    ) -> Program {
        let id = self.backend.create_program();
        self.backend.attach_shader(id, vertex.id());
        self.backend.attach_shader(id, fragment.id());
        self.backend.link_program(id);
        let tx = self.program_queue.tx();
        let mut program = Program::new(id, tx);
        for binding in 0 .. MAX_UNIFORM_BLOCKS {
            match bindings.uniform_blocks[binding] {
                UniformBlockBinding::Required(name) => {
                    let cstr = util::cstr(name);
                    let index = self
                        .query_uniform_block_index(&program, cstr)
                        .expect("missing required uniform block index");
                    self.set_uniform_block_binding(
                        &program,
                        index,
                        binding as u32,
                    );
                }
                UniformBlockBinding::None => {}
            }
        }
        for binding in 0 .. MAX_SAMPLERS {
            match bindings.samplers[binding] {
                SamplerBinding::Required(name) => {
                    let cstr = util::cstr(name);
                    let index = self
                        .query_uniform_index(&program, cstr)
                        .expect("missing required sampler index");
                    program.samplers[binding] = Some(index);
                }
                SamplerBinding::None => {}
            }
        }
        program
    }

    /// Sets the binding index for a named uniform block.
    pub fn set_uniform_block_binding(
        &self,
        program: &Program,
        index: u32,
        binding: u32,
    ) {
        self.backend.uniform_block_binding(program.id(), index, binding);
    }

    /// Retrieves the index of a named uniform block.
    pub fn query_uniform_block_index(
        &self,
        program: &Program,
        name: &ffi::CStr,
    ) -> Option<u32> {
        match self.backend.get_uniform_block_index(program.id(), name) {
            gl::INVALID_INDEX => None,
            x => Some(x),
        }
    }

    /// Retrieves the index of a named uniform.
    pub fn query_uniform_index(
        &self,
        program: &Program,
        name: &ffi::CStr,
    ) -> Option<u32> {
        match self.backend.get_uniform_location(program.id(), name) {
            -1 => None,
            x => Some(x as u32),
        }
    }

    /// Create a 2D texture backed by uninitialized GPU memory.
    pub fn texture2(
        &self,
        width: u32,
        height: u32,
        mipmap: bool,
        format: texture::Format,
    ) -> Texture2 {
        let id = self.backend.gen_texture();
        let tx = self.texture_queue.tx();
        self.backend.bind_texture(gl::TEXTURE_2D, id);
        self.backend.tex_image_2d(
            gl::TEXTURE_2D,
            format.as_gl_enum(),
            width as _,
            height as _,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            ptr::null() as _,
        );
        if mipmap {
            self.backend.generate_mipmap(gl::TEXTURE_2D);
        }
        self.backend.bind_texture(gl::TEXTURE_2D, 0);
        Texture2::new(id, width, height, mipmap, format, tx)
    }

    /// Read back the contents of a [`Texture2`].
    ///
    /// [`Texture2`]: texture/struct.Texture2.html
    pub fn read_texture2<F, T>(
        &self,
        texture: &Texture2,
        format: F,
        contents: &mut [T],
    )
        where image::Format: From<F>
    {
        self.backend.bind_texture(gl::TEXTURE_2D, texture.id());
        let (type_, format) = image::Format::from(format).as_gl_enums();
        self.backend.get_tex_image(
            gl::TEXTURE_2D,
            format,
            type_,
            contents.as_mut_ptr() as *mut _,
        );
    }

    /// (Re)-initialize the contents of a [`Texture2`].
    ///
    /// [`Texture2`]: texture/struct.Texture2.html
    pub fn write_texture2<F, T>(
        &self,
        texture: &Texture2,
        format: F,
        data: &[T],
    )
        where image::Format: From<F>
    {
        self.backend.bind_texture(gl::TEXTURE_2D, texture.id());
        let (type_, format) = image::Format::from(format).as_gl_enums();
        self.backend.tex_image_2d(
            gl::TEXTURE_2D,
            texture.format().as_gl_enum(),
            texture.width() as u32,
            texture.height() as u32,
            format,
            type_,
            data.as_ptr() as *const _,
        );
        if texture.mipmap() {
            self.backend.generate_mipmap(gl::TEXTURE_2D);
        }
        self.backend.bind_texture(gl::TEXTURE_2D, 0);
    }

    /// Create a renderbuffer.
    pub fn renderbuffer(
        &self,
        width: u32,
        height: u32,
        samples: u32,
        format: texture::Format,
    ) -> Renderbuffer {
        let id = self.backend.gen_renderbuffer();
        self.backend.bind_renderbuffer(id);
        if samples > 1 {
            self.backend.renderbuffer_storage(
                format.as_gl_enum(),
                width as _,
                height as _,
            )
        } else {
            self.backend.renderbuffer_storage_multisample(
                samples as _,
                format.as_gl_enum(),
                width as _,
                height as _,
            )
        }
        Renderbuffer::new(id)
    }

    /// Create a framebuffer.
    pub fn framebuffer(
        &self,
        width: u32,
        height: u32,
        color_attachments: [ColorAttachment; MAX_COLOR_ATTACHMENTS],
    ) -> Framebuffer {
        let id = self.backend.gen_framebuffer();
        self.backend.bind_framebuffer(id);
        let mut draw_buffers = vec![];
        for attachment in 0 .. MAX_COLOR_ATTACHMENTS {
            match color_attachments[attachment] {
                ColorAttachment::Renderbuffer(ref renderbuffer) => {
                    draw_buffers.push(gl::COLOR_ATTACHMENT0 + attachment as u32);
                    self.backend.framebuffer_renderbuffer(
                        attachment as u32,
                        renderbuffer.id(),
                    )
                }
                ColorAttachment::Texture2(ref texture2) => {
                    draw_buffers.push(gl::COLOR_ATTACHMENT0 + attachment as u32);
                    self.backend.framebuffer_texture2d(
                        attachment as u32,
                        texture2.id(),
                    )
                }
                ColorAttachment::None => {}
            }
        }
        self.backend.draw_buffers(&draw_buffers);
        Framebuffer::internal(
            id,
            width,
            height,
            color_attachments,
        )
    }

    /// Perform a draw call.
    pub fn draw(
        &self,
        framebuffer: &Framebuffer,
        state: &State,
        vertex_array: &VertexArray,
        draw_call: &DrawCall,
        invocation: &Invocation,
    ) {
        self.backend.bind_framebuffer(framebuffer.id());
        match state.viewport {
            Viewport::Max => {
                let (x, y) = (0, 0);
                let (w, h) = framebuffer.dimensions();
                self.backend.viewport(x, y, w, h)
            }
            Viewport::Subset { x, y, w, h } => {
                self.backend.viewport(x, y, w, h);
            }
        }
        if let Some(opt) = state.culling.as_gl_enum_if_enabled() {
            self.backend.enable(gl::CULL_FACE);
            self.backend.cull_face(opt);
            self.backend.front_face(state.front_face.as_gl_enum());
        } else {
            self.backend.disable(gl::CULL_FACE);
        }
        self.backend.enable(gl::DEPTH_TEST);
        self.backend.depth_func(state.depth_test.as_gl_enum());
        self.backend.bind_vertex_array(vertex_array.id());
        self.backend.use_program(invocation.program.id());
        for (idx, opt) in invocation.uniforms.iter().enumerate() {
            opt.map(|buf| {
                self.backend.bind_buffer_base(
                    gl::UNIFORM_BUFFER,
                    idx as u32,
                    buf.id(),
                );
            });
        }
        for (idx, opt) in invocation.samplers.iter().enumerate() {
            opt.map(|sampler| {
                let (id, ty) = (sampler.id(), sampler.ty());
                self.backend.active_texture(idx as u32);
                self.backend.bind_texture(ty, id);
                self.backend.tex_parameteri(
                    ty,
                    gl::TEXTURE_MAG_FILTER,
                    sampler.mag_filter.as_gl_enum(),
                );
                self.backend.tex_parameteri(
                    ty,
                    gl::TEXTURE_MIN_FILTER,
                    sampler.min_filter.as_gl_enum(),
                );
                self.backend.tex_parameteri(
                    ty,
                    gl::TEXTURE_WRAP_S,
                    sampler.wrap_s.as_gl_enum(),
                );
                self.backend.tex_parameteri(
                    ty,
                    gl::TEXTURE_WRAP_T,
                    sampler.wrap_t.as_gl_enum(),
                );
            });
        }
        self.backend.polygon_mode(gl::FRONT_AND_BACK, state.polygon_mode.as_gl_enum());
        match state.polygon_mode {
            PolygonMode::Point(size) => self.backend.point_size(size as f32),
            PolygonMode::Line(width) => self.backend.line_width(width as f32),
            PolygonMode::Fill => {},
        }
        match draw_call.kind {
            Kind::Arrays => {
                let mode = draw_call.primitive.as_gl_enum();
                self.backend.draw_arrays(mode, draw_call.offset, draw_call.count);
            },
            Kind::ArraysInstanced(_) => {
                unimplemented!()
            },
            Kind::Elements => {
                let mode = draw_call.primitive.as_gl_enum();
                let accessor = vertex_array.indices().unwrap();
                let format = accessor.format().gl_data_type();
                self.backend.draw_elements(mode, draw_call.offset, draw_call.count, format);
            },
            Kind::ElementsInstanced(_) => {
                unimplemented!()
            },
        }
        self.backend.use_program(0);
        self.backend.bind_vertex_array(0);
    }
}
