#![feature(box_syntax)]

extern crate vec_map;

mod gl;

use std::{mem, ops, os, ptr};
use std::sync::mpsc;
use std::sync::Arc;

#[doc(hidden)]
pub fn init<F>(func: F) -> gl::Gl
    where F: FnMut(&str) -> *const os::raw::c_void
{
    gl::Gl::load_with(func)
}

#[doc(inline)]
pub use buffer::Buffer;

#[doc(inline)]
pub use draw_call::DrawCall;

#[doc(inline)]
pub use program::Program;

#[doc(inline)]
pub use vertex_array::VertexArray;

struct Queue<T> {
    /// Send half of queue.
    tx: mpsc::Sender<T>,
    /// Receive half of queue.
    _rx: mpsc::Receiver<T>,
}

impl<T> Queue<T> {
    /// Constructor.
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        Self { tx, _rx: rx }
    }

    /// Clone the send half of the queue.
    pub fn tx(&self) -> mpsc::Sender<T> {
        self.tx.clone()
    }
}

impl<T> Default for Queue<T> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Factory {
    gl: gl::Gl,
    buffer_queue: Queue<buffer::Destroyed>,
    vertex_array_queue: Queue<vertex_array::Destroyed>,
    program_queue: Queue<program::Destroyed>,
}

pub mod buffer {
    use gl;
    use std::{fmt, marker, ops};
    use std::sync::{self, mpsc};

    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    pub enum Ty {
        /// Corresponds to `GL_ARRAY_BUFFER`.
        Array,

        /// Corresponds to `GL_ELEMENT_ARRAY_BUFFER`.
        Index,

        /// Corresponds to `GL_UNIFORM_BUFFER`.
        Uniform,

        /// Corresponds to `GL_TEXTURE_BUFFER`.
        Texture,
    }

    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    pub enum Usage {
        /// Corresponds to `GL_STATIC_DRAW`'
        Static,
    }

    pub(crate) struct Destroyed {
        pub id: Id,
        pub ty: Ty,
        pub sz: usize,
        pub usage: Usage,
    }

    pub type Id = u32;

    pub struct Buffer {
        pub(crate) id: Id,
        pub(crate) ty: Ty,
        pub(crate) sz: usize,
        pub(crate) usage: Usage,
        pub(crate) tx: Box<mpsc::Sender<Destroyed>>,
    }

    impl Buffer {
        pub fn id(&self) -> Id {
            self.id
        }
    }

    impl fmt::Debug for Buffer {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            #[derive(Debug)]
            struct Buffer {
                id: Id,
                ty: Ty,
                size: usize,
                usage: Usage,
            }

            Buffer {
                id: self.id,
                ty: self.ty,
                size: self.sz,
                usage: self.usage,
            }.fmt(f)
        }
    }

    impl ops::Drop for Buffer {
        fn drop(&mut self) {
            let _ = self.tx.send(Destroyed {
                id: self.id,
                ty: self.ty,
                sz: self.sz,
                usage: self.usage,
            });
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub struct Slice<'a> {
        pub(crate) id: Id,
        pub(crate) ty: Ty,
        pub(crate) off: usize,
        pub(crate) len: usize,

        _mk: &'a marker::PhantomData<()>,
    }

    #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
    pub enum Format {
        Float {
            bits: u8,
            size: u8,
        },

        Signed {
            bits: u8,
            norm: bool,
            size: u8,
        },

        Unsigned {
            bits: u8,
            norm: bool,
            size: u8,
        },
    }

    impl Format {
        pub(crate) fn gl_data_type(self) -> u32 {
            match self {
                Format::Float { bits: 32, .. } => gl::FLOAT,
                 _ => unimplemented!(),
            }
        }

        pub fn norm(self) -> bool {
             match self {
                 Format::Signed { norm, .. } => norm,
                 Format::Unsigned { norm, .. } => norm,
                 _ => false,
            }
        }

        pub fn size(self) -> usize {
            match self {
                Format::Float { size, .. } => size as usize,
                Format::Signed { size, .. } => size as usize,
                Format::Unsigned { size, .. } => size as usize,
            }
        }
    }
    
    #[derive(Clone, Debug)]
    pub struct Accessor {
        pub(crate) buffer: sync::Arc<Buffer>,
        pub(crate) format: Format,
        pub(crate) offset: usize,
        pub(crate) stride: usize,
    }

    impl Usage {
        pub fn as_gl_enum(self) -> u32 {
            match self {
                Usage::Static => gl::STATIC_DRAW,
            }
        }
    }

    impl Ty {
        pub fn as_gl_enum(self) -> u32 {
            match self {
                Ty::Array => gl::ARRAY_BUFFER,
                Ty::Index => gl::ELEMENT_ARRAY_BUFFER,
                Ty::Uniform => gl::UNIFORM_BUFFER,
                Ty::Texture => gl::TEXTURE_BUFFER,
            }
        }
    }

    impl Accessor {
        pub fn new(
            buffer: sync::Arc<Buffer>,
            format: Format,
            offset: usize,
            stride: usize,
        ) -> Self {
            Self {
                buffer,
                format,
                offset,
                stride,
            }
        }
    }

    impl Buffer {
        pub fn slice(&self, off: usize, len: usize) -> Slice {
            Slice {
                id: self.id,
                ty: self.ty,
                off,
                len,
                _mk: &marker::PhantomData,
            }
        }
    }
}

pub mod draw_call {
    use std::ops;
    use std::sync::Arc;
    use program::Program;
    use vertex_array::VertexArray;

    pub struct DrawCall {
        pub(crate) vertex_array: Arc<VertexArray>,
        pub(crate) range: ops::Range<usize>,
        pub(crate) program: Arc<Program>,
    }
}

pub mod program {
    use gl;
    use std::ops;
    use std::sync::mpsc;

    #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
    pub enum Kind {
        /// Corresponds to `GL_VERTEX_SHADER`.
        Vertex,

        /// Corresponds to `GL_FRAGMENT_SHADER`.
        Fragment,
    }

    impl Kind {
        pub fn as_gl_enum(self) -> u32 {
            match self {
                Kind::Vertex => gl::VERTEX_SHADER,
                Kind::Fragment => gl::FRAGMENT_SHADER,
            }
        }
    }
    
    pub type Source<'a> = *const i8;

    pub(crate) enum Destroyed {
        Object(u32),
        Program(u32),
    }

    /// An unlinked component of a program, i.e. a compiled
    /// vertex or fragment shader.
    pub struct Object {
        pub(crate) id: u32,
        pub(crate) kind: Kind,
        pub(crate) tx: mpsc::Sender<Destroyed>,
    }

    impl ops::Drop for Object {
        fn drop(&mut self) {
            let _ = self.tx.send(Destroyed::Object(self.id));
        }
    }

    /// A compiled shader program.
    pub struct Program {
        pub(crate) id: u32,
        pub(crate) tx: mpsc::Sender<Destroyed>,
    }

    impl ops::Drop for Program {
        fn drop(&mut self) {
            let _ = self.tx.send(Destroyed::Program(self.id));
        }
    }
}

pub mod vertex_array {
    use buffer::Accessor;
    use std::boxed::Box;
    use std::sync::mpsc;
    use vec_map::VecMap;

    pub const MAX_ATTRIBUTES: usize = 8;

    pub(crate) struct Destroyed {
        pub id: u32,
    }

    pub struct VertexArray {
        pub(crate) id: u32,
        pub(crate) indices: Option<Accessor>,
        pub(crate) attributes: VecMap<Accessor>,
        pub(crate) tx: Box<mpsc::Sender<Destroyed>>,
    }

    #[derive(Clone, Debug, Default)]
    pub struct Builder {
        pub indices: Option<Accessor>,
        pub attributes: VecMap<Accessor>,
    }

    impl VertexArray {
        pub fn builder() -> Builder {
            Builder::new()
        }
    }

    impl Builder {
        pub fn new() -> Self {
            Self {
                indices: None,
                attributes: VecMap::new(),
            }
        }

        pub fn attribute(&mut self, id: u8, accessor: Accessor) -> &mut Self {
            self.attributes.insert(id as usize, accessor);
            self
        }

        pub fn indices(&mut self, accessor: Accessor) -> &mut Self {
            self.indices = Some(accessor);
            self
        }
    }
}

impl Factory {
    pub fn new(gl: gl::Gl) -> Self {
        Self {
            gl,
            buffer_queue: Queue::new(),
            vertex_array_queue: Queue::new(),
            program_queue: Queue::new(),
        }
    }

    pub fn init<T>(&self, buffer: &Buffer, data: &[T]) {
        self.bind_buffer(buffer.id, buffer.ty.as_gl_enum());
        self.buffer_data(
            buffer.ty.as_gl_enum(),
            data.len() * mem::size_of::<T>(),
            data.as_ptr() as *const _,
            buffer.usage.as_gl_enum(),
        );
    }

    pub fn write_slice<T>(&self, slice: buffer::Slice, data: &[T]) {
        self.bind_buffer(slice.id, slice.ty.as_gl_enum());
        self.buffer_sub_data(slice.ty.as_gl_enum(), slice.off, slice.len, data.as_ptr());
    }

    // Error checking

    fn check_error(&self) {
        let error = unsafe { self.gl.GetError() };
        if error != 0 {
            panic!("OpenGL error: {}", error);
        }
    }

    // Buffer operations
    
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

    fn bind_buffer(&self, id: u32, ty: u32) {
        unsafe {
            println!("glBindBuffer{:?}", (ty, id));
            self.gl.BindBuffer(ty, id);
        }
        self.check_error();
    }

    fn buffer_data<T>(&self, id: u32, len: usize, ptr: *const T, usage: u32) {
        unsafe {
            println!("glBufferData{:?}", (id, len, ptr, usage));
            self.gl.BufferData(id, len as _, ptr as *const _, usage);
        }
        self.check_error();
    }

    fn buffer_sub_data<T>(&self, ty: u32, off: usize, len: usize, ptr: *const T) {
        unsafe {
            println!("glBufferSubData{:?}", (ty, off, len, ptr));
            self.gl.BufferSubData(ty, off as _, len as _, ptr as *const _);
        }
        self.check_error();
    }

    pub fn buffer(&self, ty: buffer::Ty, usage: buffer::Usage) -> Arc<Buffer> {
        let id = self.gen_buffer();
        let sz = 0;
        let tx = box self.buffer_queue.tx();
        Arc::new(Buffer {
            id,
            ty,
            sz,
            tx,
            usage,
        })
    }

    // Vertex array operations

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

    fn bind_vertex_array(&self, id: u32) {
        unsafe {
            println!("glBindVertexArray{:?}", (id,));
            self.gl.BindVertexArray(id);
        }
        self.check_error();
    }

    fn vertex_attrib_pointer(&self, id: u8, sz: i32, ty: u32, norm: bool, stride: i32, off: usize) {
        unsafe {
            println!("glVertexAttribPointer{:?}", (id, sz, ty, norm, stride, off));
            self.gl.VertexAttribPointer(id as _, sz as _, ty, if norm == true { 1 } else { 0 }, stride as _, off as *const _);
        }
        self.check_error();
    }

    fn enable_vertex_attrib_array(&self, idx: u8) {
        unsafe {
            println!("glEnableVertexAttribArray{:?}", (idx,));
            self.gl.EnableVertexAttribArray(idx as _);
        }
        self.check_error();
    }

    /// A collection of GPU buffers that may be drawn with a material.
    pub fn vertex_array(&self, builder: vertex_array::Builder) -> Arc<VertexArray> {
        let id = self.gen_vertex_array();
        let tx = box self.vertex_array_queue.tx();

        // Setup the vertex array
        {
            self.bind_vertex_array(id);
            if let Some(ref accessor) = builder.indices {
                self.bind_buffer(accessor.buffer.id, gl::ELEMENT_ARRAY_BUFFER);
            }
            for idx in 0 .. vertex_array::MAX_ATTRIBUTES {
                if let Some(ref accessor) = builder.attributes.get(idx) {
                    self.bind_buffer(accessor.buffer.id, gl::ARRAY_BUFFER);
                    self.enable_vertex_attrib_array(idx as _);
                    self.vertex_attrib_pointer(
                        idx as u8,
                        accessor.format.size() as _,
                        accessor.format.gl_data_type(),
                        accessor.format.norm(),
                        accessor.stride as _,
                        accessor.offset,
                    )
                }
            }
            self.bind_vertex_array(0);
        }

        Arc::new(VertexArray {
            id,
            tx,
            indices: builder.indices,
            attributes: builder.attributes,
        })
    }

    // Program operations

    fn create_shader(&self, ty: u32) -> u32 {
        let id = unsafe {
            print!("glCreateShader{:?} ", (ty,));
            self.gl.CreateShader(ty)
        };
        println!("=> {}", id);
        self.check_error();
        id
    }

    fn shader_source(&self, id: u32, source: *const i8) {
        unsafe {
            println!("glShaderSource{:?}", (id, source));
            self.gl.ShaderSource(id, 1, &source as *const _, ptr::null());
        }
        self.check_error();
    }

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

    fn create_program(&self) -> u32 {
        let id = unsafe {
            print!("glCreateProgram() ");
            self.gl.CreateProgram()
        };
        println!("=> {}", id);
        self.check_error();
        id
    }

    fn attach_shader(&self, program: u32, shader: u32) {
        unsafe {
            println!("glAttachShader{:?}", (program, shader));
            self.gl.AttachShader(program, shader);
        }
        self.check_error();
    }

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

    pub fn program_object(
        &self,
        kind: program::Kind,
        sources: program::Source,
    ) -> Arc<program::Object> {
        let id = self.create_shader(kind.as_gl_enum());
        self.shader_source(id, sources);
        self.compile_shader(id);
        let tx = self.program_queue.tx();
        Arc::new(program::Object { id, tx, kind })
    }

    pub fn program(
        &self,
        vertex: &program::Object,
        fragment: &program::Object,
    ) -> Arc<Program> {
        let id = self.create_program();
        self.attach_shader(id, vertex.id);
        self.attach_shader(id, fragment.id);
        self.link_program(id);
        let tx = self.program_queue.tx();
        Arc::new(Program { id, tx })
    }

    // Draw call operations

    fn draw_arrays(&self, mode: u32, offset: usize, count: usize) {
        unsafe {
            println!("glDrawArrays{:?}", (mode, offset, count));
            self.gl.DrawArrays(mode, offset as _, count as _);
        }
        self.check_error();
    }

    fn draw_elements(&self, mode: u32, offset: usize, count: usize, ty: u32) {
        unsafe {
            println!("glDrawElements{:?}", (mode, count, ty, offset));
            self.gl.DrawElements(mode, count as _, ty, offset as *const _);
        }
        self.check_error();
    }

    fn use_program(&self, id: u32) {
        unsafe {
            println!("glUseProgram{:?}", (id,));
            self.gl.UseProgram(id);
        }
        self.check_error();
    }

    // TODO: Move somewhere else
    pub fn draw(&self, draw_call: &DrawCall) {
        self.bind_vertex_array(draw_call.vertex_array.id);
        self.use_program(draw_call.program.id);
        if let Some(ref accessor) = draw_call.vertex_array.indices {
            self.draw_elements(
                gl::TRIANGLES,
                draw_call.range.start,
                draw_call.range.end - draw_call.range.start,
                accessor.format.gl_data_type(),
            );
        } else {
            self.draw_arrays(
                gl::TRIANGLES,
                draw_call.range.start,
                draw_call.range.end - draw_call.range.start,
            );
        }
    }

    pub fn draw_call(
        &self,
        vertex_array: Arc<VertexArray>,
        range: ops::Range<usize>,
        program: Arc<Program>,
    ) -> DrawCall {
        DrawCall {
            vertex_array,
            range,
            program,
        }
    }
}
