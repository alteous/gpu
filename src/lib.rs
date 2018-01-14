extern crate vec_map;

mod gl;

use std::{ffi, mem, ops, os, ptr};
use std::sync::mpsc;

#[doc(hidden)]
pub fn init<F>(func: F) -> gl::Gl
    where F: FnMut(&str) -> *const os::raw::c_void
{
    gl::Gl::load_with(func)
}

#[doc(inline)]
pub use buffer::Buffer;

#[doc(inline)]
pub use program::Program;

#[doc(inline)]
pub use vertex_array::VertexArray;

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

/// GPU buffer management.
pub mod buffer {
    use gl;
    use std::{cmp, fmt, hash, ops};
    use std::sync::{self, mpsc};

    /// OpenGL buffer ID type.
    pub(crate) type Id = u32;

    /// Determines what the buffer may be used for.
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    pub enum Kind {
        /// Corresponds to `GL_ARRAY_BUFFER`.
        Array,

        /// Corresponds to `GL_ELEMENT_ARRAY_BUFFER`.
        Index,

        /// Corresponds to `GL_UNIFORM_BUFFER`.
        Uniform,

        /// Corresponds to `GL_TEXTURE_BUFFER`.
        Texture,
    }

    impl Kind {
        /// Returns the equivalent OpenGL usage enumeration constant.
        pub fn as_gl_enum(self) -> u32 {
            match self {
                Kind::Array => gl::ARRAY_BUFFER,
                Kind::Index => gl::ELEMENT_ARRAY_BUFFER,
                Kind::Uniform => gl::UNIFORM_BUFFER,
                Kind::Texture => gl::TEXTURE_BUFFER,
            }
        }
    }

    /// A buffer data usage hint.
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    pub enum Usage {
        /// Corresponds to `GL_STATIC_DRAW`.
        StaticDraw,

        /// Corresponds to `GL_DYNAMIC_DRAW`.
        DynamicDraw,
    }

    impl Usage {
        /// Returns the equivalent OpenGL usage enumeration constant.
        pub(crate) fn as_gl_enum(self) -> u32 {
            match self {
                Usage::StaticDraw => gl::STATIC_DRAW,
                Usage::DynamicDraw => gl::DYNAMIC_DRAW,
            }
        }
    }
    
    /// Pushes the buffer ID onto the factory buffer queue when destroyed.
    pub(crate) struct Destructor {
        id: Id,
        tx: mpsc::Sender<Id>,
    }

    impl ops::Drop for Destructor {
        fn drop(&mut self) {
            let _ = self.tx.send(self.id);
        }
    }

    /// A contiguous region of GPU memory.
    #[derive(Clone)]
    pub struct Buffer {
        /// The OpenGL buffer ID.
        id: Id,

        /// The type of buffer, e.g. a vertex buffer.
        kind: Kind,

        /// The number of bytes held by the buffer.
        size: usize,

        /// Data usage hint.
        usage: Usage,

        /// Returns the buffer back to the factory upon destruction.
        destructor: sync::Arc<Destructor>,
    }

    impl Buffer {
        /// Constructor.
        pub(crate) fn new(
            id: Id,
            kind: Kind,
            size: usize,
            usage: Usage,
            tx: mpsc::Sender<Id>,
        ) -> Self {
            Self {
                destructor: sync::Arc::new(Destructor { id, tx }),
                id,
                kind,
                size,
                usage,
            }
        
        }

        /// Returns the OpenGL buffer ID.
        pub(crate) fn id(&self) -> Id {
            self.id
        }

        /// Returns the buffer kind.
        pub fn kind(&self) -> Kind {
            self.kind
        }

        /// Returns the number of bytes this buffer contains.
        pub fn size(&self) -> usize {
            self.size
        }

        /// Returns the buffer data usage hint.
        pub fn usage(&self) -> Usage {
            self.usage
        }
 
        /// Creates a slice into a [`Buffer`].
        ///
        /// [`Buffer`]: struct.Buffer.html
        pub fn slice(&self, offset: usize, length: usize) -> Slice {
            Slice::new(self, offset, length)
        }
    }

    impl cmp::PartialEq<Self> for Buffer {
        fn eq(&self, other: &Self) -> bool {
            self.id == other.id
        }
    }

    impl cmp::Eq for Buffer {}

    impl fmt::Debug for Buffer {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            #[derive(Debug)]
            struct Buffer {
                id: Id,
                kind: Kind,
                size: usize,
                usage: Usage,
            }

            Buffer {
                id: self.id,
                kind: self.kind,
                size: self.size,
                usage: self.usage,
            }.fmt(f)
        }
    }

    impl hash::Hash for Buffer {
        fn hash<H: hash::Hasher>(&self, state: &mut H) {
            self.id.hash(state);
        }
    }

    #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
    pub struct Slice<'a> {
        buffer: &'a Buffer,
        offset: usize,
        length: usize,
    }

    impl<'a> Slice<'a> {
        /// Constructor.
        pub fn new(buffer: &'a Buffer, offset: usize, length: usize) -> Self {
            Self { buffer, offset, length }
        }

        /// Returns the parent buffer OpenGL ID.
        pub(crate) fn id(&self) -> Id {
            self.buffer.id
        }

        /// Returns the parent buffer.
        pub fn buffer(&self) -> &Buffer {
            self.buffer
        }

        /// Returns the parent buffer kind.
        pub fn kind(&self) -> Kind {
            self.buffer.kind
        }

        /// Returns the byte offset into the parent buffer.
        pub fn offset(&self) -> usize {
            self.offset
        }

        /// Returns the number of bytes accessed from the offset.
        pub fn length(&self) -> usize {
            self.length
        }
    }

    /// Describes the data format of an individual item in an accessor.
    #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
    pub enum Format {
        /// Specifies a floating point type.
        Float {
            /// The number of bits in each element, e.g. 32.
            bits: u8,
            /// The number of elements, e.g. 3 for `vec3`.
            size: u8,
        },

        /// Specifies a signed integer type.
        Signed {
            /// The number of bits in each element, e.g. 32.
            bits: u8,
            /// Specifies whether this is a normalized integer type.
            norm: bool,
            /// The number of elements, e.g. 3 for `ivec3` or `vec3`.
            size: u8,
        },

        /// Specifies an unsigned integer type.
        Unsigned {
            /// The number of bits in each element, e.g. 32.
            bits: u8,
            /// Specifies whether this is a normalized integer type.
            norm: bool,
            /// The number of elements, e.g. 3 for `uvec3` or `vec3`.
            size: u8,
        },
    }

    impl Format {
        /// Returns the corresponding GL data type enumeration constant.
        pub(crate) fn gl_data_type(self) -> u32 {
            match self {
                Format::Float { bits: 32, .. } => gl::FLOAT,
                Format::Signed { bits: 8, .. } => gl::BYTE,
                Format::Signed { bits: 16, .. } => gl::SHORT,
                Format::Signed { bits: 32, .. } => gl::INT,
                Format::Unsigned { bits: 8, .. } => gl::UNSIGNED_BYTE,
                Format::Unsigned { bits: 16, .. } => gl::UNSIGNED_SHORT,
                Format::Unsigned { bits: 32, .. } => gl::UNSIGNED_INT,
                _ => panic!("Invalid GL format {:?}", self),
            }
        }

        /// Returns the width of each element in bits.
        pub fn bits(self) -> u8 {
            match self {
                Format::Signed { bits, .. } => bits,
                Format::Unsigned { bits, .. } => bits,
                Format::Float { bits, .. } => bits,
            }
        }

        /// Returns true if this is a normalized type.
        pub fn norm(self) -> bool {
            match self {
                Format::Signed { norm, .. } => norm,
                Format::Unsigned { norm, .. } => norm,
                _ => false,
            }
        }

        /// Returns the number of elements.
        pub fn size(self) -> usize {
            match self {
                Format::Float { size, .. } => size as usize,
                Format::Signed { size, .. } => size as usize,
                Format::Unsigned { size, .. } => size as usize,
            }
        }
    }

    /// A formatted view into a [`Buffer`].
    ///
    /// [`Buffer`]: struct.Buffer.html
    #[derive(Clone, Debug)]
    pub struct Accessor {
        /// The buffer the accessor reads from.
        buffer: Buffer,

        /// The accessor data format.
        format: Format,

        /// The number of bytes into the buffer the accessor reads from.
        offset: usize,

        /// The number of bytes between each element.
        stride: usize,
    }

    impl Accessor {
        /// Constructor.
        pub fn new(
            buffer: Buffer,
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

        /// Returns the parent buffer.
        pub fn buffer(&self) -> &Buffer {
            &self.buffer
        }

        /// Returns the accessor data format.
        pub fn format(&self) -> Format {
            self.format
        }

        /// Returns the accessor byte offset into the parent buffer.
        pub fn offset(&self) -> usize {
            self.offset
        }

        /// Returns the accessor byte stride between consecutive elements.
        pub fn stride(&self) -> usize {
            self.stride
        }
    }
}

/// GLSL programs.
pub mod program {
    use gl;
    use std::{cmp, ffi, fmt, hash, ops};
    use std::sync::{self, mpsc};

    use buffer::Buffer;

    /// Specifies the maximum number of uniforms permitted by the crate.
    pub const MAX_UNIFORMS: usize = 4;

    /// The program source code type.
    pub type Source = ffi::CStr;
    
    /// Determines the shader type, e.g. a vertex or fragment shader.
    #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
    pub enum Kind {
        /// Corresponds to `GL_VERTEX_SHADER`.
        Vertex,

        /// Corresponds to `GL_FRAGMENT_SHADER`.
        Fragment,
    }

    impl Kind {
        /// Returns the equivalent OpenGL shader enumeration constant.
        pub(crate) fn as_gl_enum(self) -> u32 {
            match self {
                Kind::Vertex => gl::VERTEX_SHADER,
                Kind::Fragment => gl::FRAGMENT_SHADER,
            }
        }
    }

    /// Specifies whether the destroyed item was an object or a program.
    pub(crate) enum Destroyed {
        /// A shader object.
        Object(u32),

        /// A compiled and linked program.
        Program(u32),
    }

    /// Pushes the shader/program ID onto the factory program queue when
    /// destroyed.
    pub(crate) struct ObjectDestructor {
        id: u32,
        tx: mpsc::Sender<Destroyed>,
    }

    impl ops::Drop for ObjectDestructor {
        fn drop(&mut self) {
            let _ = self.tx.send(Destroyed::Object(self.id));
        }
    
    }

    /// Pushes the shader/program ID onto the factory program queue when
    /// destroyed.
    pub(crate) struct ProgramDestructor {
        id: u32,
        tx: mpsc::Sender<Destroyed>,
    }

    impl ops::Drop for ProgramDestructor {
        fn drop(&mut self) {
            let _ = self.tx.send(Destroyed::Program(self.id));
        }
    }

    /// An unlinked component of a GLSL program, e.g. a compiled
    /// vertex or fragment shader.
    pub struct Object {
        /// The OpenGL shader object ID.
        id: u32,

        /// Determines the shader type, e.g. a vertex or fragment shader.
        kind: Kind,

        /// Returns the object back to the factory upon destruction.
        destructor: sync::Arc<ObjectDestructor>,
    }

    impl Object {
        /// Constructor.
        pub(crate) fn new(
            id: u32,
            kind: Kind,
            tx: mpsc::Sender<Destroyed>,
        ) -> Self {
            Self {
                destructor: sync::Arc::new(
                    ObjectDestructor {
                        id,
                        tx,
                    },
                ),
                id,
                kind,
            }
        }

        /// Returns the GLSL object ID.
        pub(crate) fn id(&self) -> u32 {
            self.id
        }
    }

    impl cmp::Eq for Object {}

    impl cmp::PartialEq<Self> for Object {
        fn eq(&self, other: &Self) -> bool {
            self.id == other.id
        }
    }

    impl fmt::Debug for Object {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            #[derive(Debug)]
            struct Object(u32, Kind);

            Object(self.id, self.kind).fmt(f)
        }
    }

    impl hash::Hash for Object {
        fn hash<H: hash::Hasher>(&self, state: &mut H) {
            self.id.hash(state)
        }
    }

    /// An invocation of a shader program.
    pub struct Invocation<'a> {
        /// The program to bind at draw time.
        pub program: &'a Program,

        /// Uniform buffers to be bound to the program at draw time.
        pub uniforms: [Option<Buffer>; MAX_UNIFORMS],
    }

    /// A compiled shader program.
    pub struct Program {
        /// The OpenGL program ID.
        id: u32,

        /// Returns the program back to the factory upon destruction.
        destructor: sync::Arc<ProgramDestructor>,
    }

    impl Program {
        /// Constructor.
        pub(crate) fn new(
            id: u32,
            tx: mpsc::Sender<Destroyed>,
        ) -> Self {
            Self {
                destructor: sync::Arc::new(
                    ProgramDestructor {
                        id,
                        tx,
                    },
                ),
                id,
            }
        }

        /// Returns the GLSL program ID.
        pub(crate) fn id(&self) -> u32 {
            self.id
        }
    }

    impl cmp::Eq for Program {}

    impl cmp::PartialEq<Self> for Program {
        fn eq(&self, other: &Self) -> bool {
            self.id == other.id
        }
    }
    
    impl fmt::Debug for Program {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            #[derive(Debug)]
            struct Program(u32);

            Program(self.id).fmt(f)
        }
    }

    impl hash::Hash for Program {
        fn hash<H: hash::Hasher>(&self, state: &mut H) {
            self.id.hash(state)
        }
    }
}

/// Vertex array objects.
pub mod vertex_array {
    use buffer::Accessor;
    use std::{cmp, fmt, hash, ops};
    use std::sync::{self, mpsc};
    use vec_map::VecMap;

    /// The maximum number of vertex attributes permitted by the crate.
    pub const MAX_ATTRIBUTES: usize = 8;

    /// The OpenGL VAO ID type.
    pub(crate) type Id = u32;

    /// Returns the VAO back to the factory upon destruction.
    struct Destructor {
        id: u32,
        tx: mpsc::Sender<Id>
    }

    impl ops::Drop for Destructor {
        fn drop(&mut self) {
            let _ = self.tx.send(self.id);
        }
    }

    /// Corresponds to an OpenGL vertex array object.
    #[derive(Clone)]
    pub struct VertexArray {
        /// The OpenGL VAO ID.
        id: Id,
        
        /// Draw sequence indices to bind at draw time.
        indices: Option<Accessor>,

        /// Vertex attributes to bind at draw time.
        attributes: VecMap<Accessor>,

        /// Returns the VAO back to the factory upon destruction.
        destructor: sync::Arc<Destructor>,
    }

    impl VertexArray {
        /// Constructor.
        pub(crate) fn new(
            id: Id,
            builder: Builder,
            tx: mpsc::Sender<Id>,
        ) -> Self {
            Self {
                id,
                indices: builder.indices,
                attributes: builder.attributes,
                destructor: sync::Arc::new(Destructor { id, tx }),
            }
        }

        /// Returns the OpenGL VAO ID.
        pub(crate) fn id(&self) -> Id {
            self.id
        }

        /// Returns the accessor bound as the element array buffer.
        pub fn indices(&self) -> Option<&Accessor> {
            self.indices.as_ref()
        }

        /// Returns the accessor bound to the given attribute index.
        pub fn attribute(&self, index: u8) -> Option<&Accessor> {
            self.attributes.get(index as usize)
        }
    }

    impl cmp::Eq for VertexArray {}

    impl cmp::PartialEq<Self> for VertexArray {
        fn eq(&self, other: &Self) -> bool {
            self.id == other.id
        }
    }
    
    impl fmt::Debug for VertexArray {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            #[derive(Debug)]
            struct VertexArray<'a> {
                id: u32,
                indices: &'a Option<Accessor>,
                attributes: &'a VecMap<Accessor>,
            }

            VertexArray {
                id: self.id,
                indices: &self.indices,
                attributes: &self.attributes,
            }.fmt(f)
        }
    }

    impl hash::Hash for VertexArray {
        fn hash<H: hash::Hasher>(&self, state: &mut H) {
            self.id.hash(state)
        }
    }

    /// A vertex array object definition.
    #[derive(Clone, Debug, Default)]
    pub struct Builder {
        /// Draw sequence indices to bind at draw time.
        pub indices: Option<Accessor>,

        /// Vertex attributes to bind at draw time.
        pub attributes: VecMap<Accessor>,
    }

    impl VertexArray {
        /// Begin building a new vertex array object.
        pub fn builder() -> Builder {
            Builder::new()
        }
    }

    impl Builder {
        /// Constructor.
        pub fn new() -> Self {
            Self {
                indices: None,
                attributes: VecMap::new(),
            }
        }

        /// Bind an accessor to the given attribute index.
        pub fn attribute(&mut self, id: u8, accessor: Accessor) -> &mut Self {
            self.attributes.insert(id as usize, accessor);
            self
        }

        /// Bind an accessor as the index draw sequence.
        pub fn indices(&mut self, accessor: Accessor) -> &mut Self {
            self.indices = Some(accessor);
            self
        }
    }
}

impl Factory {
    /// Constructor
    #[doc(hidden)]
    pub fn new(gl: gl::Gl) -> Self {
        Self {
            gl,
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
