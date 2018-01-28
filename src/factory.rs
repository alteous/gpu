//! Factory.

#![allow(dead_code)]

use buffer;
use gl;
use program;
use std::{ffi, mem, os};
use texture;
use vertex_array;

use draw_call::{DrawCall, Mode};
use framebuffer::Framebuffer;
use program::Invocation;
use pipeline::State;
use queue::Queue;
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
        where F: FnMut(&str) -> *const os::raw::c_void
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
    pub fn clear_color_buffer(&self, r: f32, g: f32, b: f32, a: f32) {
        self.backend.clear_color(r, g, b, a);
    }

    /// Clear the depth buffer.
    pub fn clear_depth_buffer(&self) {
        self.backend.clear_depth();
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
    pub fn vertex_array(&self, builder: vertex_array::Builder) -> VertexArray {
        let id = self.backend.gen_vertex_array();
        let tx = self.vertex_array_queue.tx();

        // Setup the vertex array
        {
            self.backend.bind_vertex_array(id);
            if let Some(ref accessor) = builder.indices {
                self.backend.bind_buffer(accessor.buffer().id(), gl::ELEMENT_ARRAY_BUFFER);
            }
            for idx in 0 .. vertex_array::MAX_ATTRIBUTES {
                if let Some(ref accessor) = builder.attributes.get(idx) {
                    self.backend.bind_buffer(accessor.buffer().id(), gl::ARRAY_BUFFER);
                    self.backend.enable_vertex_attrib_array(idx as _);
                    self.backend.vertex_attrib_pointer(
                        idx as u8,
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

        VertexArray::new(id, builder, tx)
    }

    /// Compile a GLSL object.
    pub fn program_object(
        &self,
        kind: program::Kind,
        sources: &program::Source,
    ) -> program::Object {
        let id = self.backend.create_shader(kind.as_gl_enum());
        self.backend.shader_source(id, sources);
        self.backend.compile_shader(id);
        let tx = self.program_queue.tx();
        program::Object::new(id, kind, tx)
    }

    /// Link GLSL objects to create a GLSL program.
    pub fn program(
        &self,
        vertex: &program::Object,
        fragment: &program::Object,
    ) -> Program {
        let id = self.backend.create_program();
        self.backend.attach_shader(id, vertex.id());
        self.backend.attach_shader(id, fragment.id());
        self.backend.link_program(id);
        let tx = self.program_queue.tx();
        Program::new(id, tx)
    }

    /// Sets the binding index for a named uniform block.
    pub fn set_uniform_block_binding(
        &self,
        program: &Program,
        name: &ffi::CStr,
        binding: u32,
    ) {
        if let Some(index) = self.query_uniform_block_index(program, name) {
            self.backend.uniform_block_binding(program.id(), index, binding);
        }
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

    /// Create an uninitialized 2D texture.
    pub fn texture2(&self, builder: texture::Builder) -> Texture2 {
        let id = self.backend.gen_texture();
        let tx = self.texture_queue.tx();
        self.backend.bind_texture(gl::TEXTURE_2D, id);
        self.backend.tex_parameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, builder.min_filter.as_gl_enum());
        self.backend.tex_parameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, builder.mag_filter.as_gl_enum());
        self.backend.tex_parameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, builder.wrap_s.as_gl_enum());
        self.backend.tex_parameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, builder.wrap_t.as_gl_enum());
        self.backend.bind_texture(gl::TEXTURE_2D, 0);
        Texture2::new(id, tx)
    }

    /// (Re)-initialize the contents of a [`Texture2`].
    ///
    /// [`Texture2`]: texture/struct.Texture2.html
    pub fn initialize_texture2<T>(
        &self,
        texture: &Texture2,
        generate_mipmap: bool,
        internal_format: u32,
        width: u32,
        height: u32,
        format: u32,
        ty: u32,
        data: &[T],
    ) {
        self.backend.bind_texture(gl::TEXTURE_2D, texture.id());
        let (level, border) = (0, 0);
        self.backend.tex_image_2d(
            gl::TEXTURE_2D,
            level,
            internal_format,
            width,
            height,
            border,
            format,
            ty,
            data.as_ptr(),
        );
        if generate_mipmap {
            self.backend.generate_mipmap(gl::TEXTURE_2D);
        }
        self.backend.bind_texture(gl::TEXTURE_2D, 0);
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
        self.backend.bind_framebuffer(gl::FRAMEBUFFER, framebuffer.id());
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
        for &(idx, buf) in &invocation.uniforms {
            self.backend.bind_buffer_base(gl::UNIFORM_BUFFER, idx, buf.id());
        }
        for &(idx, sampler) in &invocation.samplers {
            self.backend.active_texture(idx);
            self.backend.bind_texture(sampler.ty(), sampler.id());
        }
        match draw_call.mode {
            Mode::Arrays => {
                let mode = draw_call.primitive.as_gl_enum();
                self.backend.draw_arrays(mode, draw_call.offset, draw_call.count);
            },
            Mode::ArraysInstanced(_) => {
                unimplemented!()
            },
            Mode::Elements => {
                let mode = draw_call.primitive.as_gl_enum();
                let accessor = vertex_array.indices().unwrap();
                let format = accessor.format().gl_data_type();
                self.backend.draw_elements(mode, draw_call.offset, draw_call.count, format);
            },
            Mode::ElementsInstanced(_) => {
                unimplemented!()
            },
        }
        self.backend.use_program(0);
        self.backend.bind_vertex_array(0);
    }
}
