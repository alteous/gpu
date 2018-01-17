//! GPU buffer management.

use gl;
use queue;
use std::{cmp, fmt, hash, ops, sync};

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
    tx: queue::Sender<Id>,
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
        tx: queue::Sender<Id>,
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
