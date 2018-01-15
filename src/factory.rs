//! Factory.

use buffer;
use gl;
use program;
use std::{ffi, mem, ops, os, ptr};
use std::sync::mpsc;
use vertex_array;

use {Buffer, Program, VertexArray};

/// A thread-safe queue.
struct Queue<T> {
    /// Send half of queue.
    tx: mpsc::Sender<T>,

    /// Receive half of queue.
    rx: mpsc::Receiver<T>,
}

impl<T> Queue<T> {
    /// Constructor.
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        Self { tx, rx }
    }

    /// Clone the send half of the queue.
    pub fn tx(&self) -> mpsc::Sender<T> {
        self.tx.clone()
    }

    /// Remove the item from the front of the queue.
    pub fn next(&self) -> Option<T> {
        self.rx.try_recv().ok()
    }
}

impl<T> Default for Queue<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// OpenGL memory manager.
pub struct Factory {
    /// Function pointers to the OpenGL backend.
    gl: gl::Gl,

    /// Destroyed buffers arrive here to be destroyed or recycled.
    buffer_queue: Queue<buffer::Id>,

    /// Destroyed vertex arrays arrive here to be destroyed or recycled.
    vertex_array_queue: Queue<vertex_array::Id>,

    /// Destroyed GLSL programs arrive here to be destroyed or recycled.
    program_queue: Queue<program::Destroyed>,
}

impl Factory {
    /// Constructor.
    pub fn new<F>(func: F) -> Self
        where F: FnMut(&str) -> *const os::raw::c_void
    {
        Self {
            gl: gl::Gl::load_with(func),
            buffer_queue: Queue::new(),
            vertex_array_queue: Queue::new(),
            program_queue: Queue::new(),
        }
    }

    /// (Re)-initialize the contents of a [`Buffer`] with the given slice.
    ///
    /// [`Buffer`]: buffer/struct.Buffer.html
    pub fn init<T>(&self, buffer: &Buffer, data: &[T]) {
        self.bind_buffer(buffer.id(), buffer.kind().as_gl_enum());
        self.buffer_data(
            buffer.kind().as_gl_enum(),
            data.len() * mem::size_of::<T>(),
            data.as_ptr() as *const _,
            buffer.usage().as_gl_enum(),
        );
    }

    /// Overwrite part of a buffer.
    pub fn overwrite<T>(&self, slice: buffer::Slice, data: &[T]) {
        self.bind_buffer(slice.id(), slice.kind().as_gl_enum());
        self.buffer_sub_data(slice.kind().as_gl_enum(), slice.offset(), slice.length(), data.as_ptr());
    }

    // Error checking

    /// Corresponds to `glGetError` plus an error check.
    fn check_error(&self) {
        let error = unsafe { self.gl.GetError() };
        if error != 0 {
            panic!("OpenGL error: {}", error);
        }
    }

    // Buffer operations

    /// Corresponds to `glGenBuffer`.
    fn gen_buffer(&self) -> u32 {
        let mut id: u32 = 0;
        unsafe {
            print!("glGenBuffers(1) ");
            self.gl.GenBuffers(1, &mut id as *mut _)
        };
        println!(" => {}", id);
        self.check_error();
        id
    }

    /// Corresponds to `glBindBuffer`.
    fn bind_buffer(&self, id: u32, ty: u32) {
        unsafe {
            println!("glBindBuffer{:?}", (ty, id));
            self.gl.BindBuffer(ty, id);
        }
        self.check_error();
    }

    /// Corresponds to `glBufferData`.
    fn buffer_data<T>(&self, id: u32, len: usize, ptr: *const T, usage: u32) {
        unsafe {
            println!("glBufferData{:?}", (id, len, ptr, usage));
            self.gl.BufferData(id, len as _, ptr as *const _, usage);
        }
        self.check_error();
    }

    /// Corresponds to `glBufferSubData`.
    fn buffer_sub_data<T>(&self, ty: u32, off: usize, len: usize, ptr: *const T) {
        unsafe {
            println!("glBufferSubData{:?}", (ty, off, len, ptr));
            self.gl.BufferSubData(ty, off as _, len as _, ptr as *const _);
        }
        self.check_error();
    }

    /// Create an uninitialized GPU buffer.
    pub fn buffer(&self, kind: buffer::Kind, usage: buffer::Usage) -> Buffer {
        let id = self.gen_buffer();
        let size = 0;
        let tx = self.buffer_queue.tx();
        Buffer::new(id, kind, size, usage, tx)
    }

    // Vertex array operations

    /// Corresponds to `glGenVertexArrays`.
    fn gen_vertex_array(&self) -> u32 {
        let mut id: u32 = 0;
        unsafe {
            print!("glGenVertexArrays(1) ");
            self.gl.GenVertexArrays(1, &mut id as *mut _);
            println!("=> {}", id);
        }
        self.check_error();
        id
    }

    /// Corresponds to `glBindVertexArray`.
    fn bind_vertex_array(&self, id: u32) {
        unsafe {
            println!("glBindVertexArray{:?}", (id,));
            self.gl.BindVertexArray(id);
        }
        self.check_error();
    }

    /// Corresponds to `glVertexAttribPointer`.
    fn vertex_attrib_pointer(&self, id: u8, sz: i32, ty: u32, norm: bool, stride: i32, off: usize) {
        unsafe {
            println!("glVertexAttribPointer{:?}", (id, sz, ty, norm, stride, off));
            self.gl.VertexAttribPointer(id as _, sz as _, ty, if norm == true { 1 } else { 0 }, stride as _, off as *const _);
        }
        self.check_error();
    }

    /// Corresponds to `glEnableVertexAttribArray`.
    fn enable_vertex_attrib_array(&self, idx: u8) {
        unsafe {
            println!("glEnableVertexAttribArray{:?}", (idx,));
            self.gl.EnableVertexAttribArray(idx as _);
        }
        self.check_error();
    }

    /// A collection of GPU buffers that may be drawn with a material.
    pub fn vertex_array(&self, builder: vertex_array::Builder) -> VertexArray {
        let id = self.gen_vertex_array();
        let tx = self.vertex_array_queue.tx();

        // Setup the vertex array
        {
            self.bind_vertex_array(id);
            if let Some(ref accessor) = builder.indices {
                self.bind_buffer(accessor.buffer().id(), gl::ELEMENT_ARRAY_BUFFER);
            }
            for idx in 0 .. vertex_array::MAX_ATTRIBUTES {
                if let Some(ref accessor) = builder.attributes.get(idx) {
                    self.bind_buffer(accessor.buffer().id(), gl::ARRAY_BUFFER);
                    self.enable_vertex_attrib_array(idx as _);
                    self.vertex_attrib_pointer(
                        idx as u8,
                        accessor.format().size() as _,
                        accessor.format().gl_data_type(),
                        accessor.format().norm(),
                        accessor.stride() as _,
                        accessor.offset(),
                    )
                }
            }
            self.bind_vertex_array(0);
        }

        VertexArray::new(id, builder, tx)
    }

    // Program operations

    /// Corresponds to `glCreateShader`.
    fn create_shader(&self, ty: u32) -> u32 {
        let id = unsafe {
            print!("glCreateShader{:?} ", (ty,));
            self.gl.CreateShader(ty)
        };
        println!("=> {}", id);
        self.check_error();
        id
    }

    /// Corresponds to `glShaderSource`.
    fn shader_source(&self, id: u32, source: &ffi::CStr) {
        unsafe {
            println!("glShaderSource{:?}", (id, source));
            let ptr = source.as_ptr() as *const i8;
            self.gl.ShaderSource(id, 1, &ptr as *const _, ptr::null());
        }
        self.check_error();
    }

    /// Corresponds to `glCompileShader`.
    fn compile_shader(&self, id: u32) {
        let mut status = 0i32;
        unsafe {
            println!("glCompileShader{:?}", (id,));
            self.gl.CompileShader(id);
            self.check_error();
            self.gl.GetShaderiv(id, gl::COMPILE_STATUS, &mut status as *mut _);
            self.check_error();
            if status == 0 {
                panic!("Shader compilation failed");
            }
        }
    }

    /// Corresponds to `glCreateProgram`.
    fn create_program(&self) -> u32 {
        let id = unsafe {
            print!("glCreateProgram() ");
            self.gl.CreateProgram()
        };
        println!("=> {}", id);
        self.check_error();
        id
    }

    /// Corresponds to `glAttachShader`.
    fn attach_shader(&self, program: u32, shader: u32) {
        unsafe {
            println!("glAttachShader{:?}", (program, shader));
            self.gl.AttachShader(program, shader);
        }
        self.check_error();
    }

    /// Corresponds to `glLinkProgram`.
    fn link_program(&self, id: u32) {
        let mut status = 0i32;
        unsafe {
            println!("glLinkProgram{:?}", (id,));
            self.gl.LinkProgram(id);
            self.check_error();
            print!("glGetProgramiv{:?} ", (id, gl::LINK_STATUS));
            self.gl.GetProgramiv(id, gl::LINK_STATUS, &mut status as *mut _);
            println!("=> {}", status);
            self.check_error();
            if status == 0i32 {
                panic!("Program linking failed");
            }
        }
    }

    /// Compile a GLSL object.
    pub fn program_object(
        &self,
        kind: program::Kind,
        sources: &program::Source,
    ) -> program::Object {
        let id = self.create_shader(kind.as_gl_enum());
        self.shader_source(id, sources);
        self.compile_shader(id);
        let tx = self.program_queue.tx();
        program::Object::new(id, kind, tx)
    }

    /// Corresponds to `glGetUniformBlockIndex`.
    fn get_uniform_block_index(
        &self,
        id: u32,
        name: &ffi::CStr,
    ) -> u32 {
        let index;
        unsafe {
            println!("glGetUniformBlockIndex{:?} ", (id, name));
            index = self.gl.GetUniformBlockIndex(id, name.as_ptr() as _);
            println!("=> {}", index);
        }
        self.check_error();
        index
    }

    /// Link GLSL objects to create a GLSL program.
    pub fn program(
        &self,
        vertex: &program::Object,
        fragment: &program::Object,
    ) -> Program {
        let id = self.create_program();
        self.attach_shader(id, vertex.id());
        self.attach_shader(id, fragment.id());
        self.link_program(id);
        let tx = self.program_queue.tx();
        Program::new(id, tx)
    }

    /// Retrieves the index of a named uniform block.
    pub fn query_uniform_block_index(
        &self,
        program: &Program,
        name: &ffi::CStr,
    ) -> Option<u32> {
        match self.get_uniform_block_index(program.id(), name) {
            gl::INVALID_INDEX => None,
            x => Some(x),
        }
    }

    // Draw call operations

    /// Corresponds to `glDrawArrays`.
    fn draw_arrays(&self, mode: u32, offset: usize, count: usize) {
        unsafe {
            println!("glDrawArrays{:?}", (mode, offset, count));
            self.gl.DrawArrays(mode, offset as _, count as _);
        }
        self.check_error();
    }

    /// Corresponds to `glDrawElements`.
    fn draw_elements(&self, mode: u32, offset: usize, count: usize, ty: u32) {
        unsafe {
            println!("glDrawElements{:?}", (mode, count, ty, offset));
            self.gl.DrawElements(mode, count as _, ty, offset as *const _);
        }
        self.check_error();
    }

    /// Corresponds to `glUseProgram`.
    fn use_program(&self, id: u32) {
        unsafe {
            println!("glUseProgram{:?}", (id,));
            self.gl.UseProgram(id);
        }
        self.check_error();
    }

    /// Corresponds to `glBindBufferBase`.
    fn bind_buffer_base(&self, target: u32, binding: u8, id: u32) {
        unsafe {
            println!("glBindBufferBase{:?}", (target, binding, id));
            self.gl.BindBufferBase(target, binding as _, id);
        }
        self.check_error();
    }

    /// Perform a draw call.
    pub fn draw(
        &self,
        vertex_array: &VertexArray,
        range: ops::Range<usize>,
        invocation: &program::Invocation,
    ) {
        self.bind_vertex_array(vertex_array.id());
        self.use_program(invocation.program.id());
        for i in 0 .. program::MAX_UNIFORMS {
            if let Some(ref buffer) = invocation.uniforms[i] {
                self.bind_buffer_base(gl::UNIFORM_BUFFER, i as _, buffer.id());
            }
        }
        if let Some(accessor) = vertex_array.indices() {
            self.draw_elements(
                gl::TRIANGLES,
                range.start,
                range.end - range.start,
                accessor.format().gl_data_type(),
            );
        } else {
            self.draw_arrays(
                gl::TRIANGLES,
                range.start,
                range.end - range.start,
            );
        }
    }
}
