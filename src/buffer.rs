//! GPU buffer management.

use gl;
use queue;
use std::{cmp, fmt, hash, ops, sync};

#[doc(inline)]
pub use self::format::Format;

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

    /// Sets the buffer size.
    pub(crate) fn set_size(&mut self, size: usize) {
        self.size = size;
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

    /// Returns a [`Slice`] convering the whole buffer.
    ///
    /// [`Slice`]: struct.Slice.html
    pub fn as_slice(&self) -> Slice {
        Slice::new(self, 0, self.size)
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

/// A contiguous sub-region of a [`Buffer`].
///
/// [`Buffer`]: struct.Buffer.html
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

/// Buffer format descriptors.
pub mod format {
    use gl;

    /// 32-bit floating point number.
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    pub struct F32(pub u8);

    /// Signed 8-bit integer.
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    pub struct I8(pub u8);

    /// Signed normalized 8-bit rational.
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    pub struct I8Norm(pub u8);

    /// Signed 16-bit integer.
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    pub struct I16(pub u8);

    /// Signed normalized 16-bit rational.
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    pub struct I16Norm(pub u8);

    /// Signed 32-bit integer.
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    pub struct I32(pub u8);

    /// Unsigned 8-bit integer.
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    pub struct U8(pub u8);

    /// Unsigned normalized 8-bit rational.
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    pub struct U8Norm(pub u8);

    /// Unsigned 16-bit integer.
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    pub struct U16(pub u8);

    /// Unsigned normalized 16-bit rational.
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    pub struct U16Norm(pub u8);

    /// Unsigned 32-bit integer.
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    pub struct U32(pub u8);

    /// Describes the data format of an individual item in an accessor.
    #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
    pub enum Format {
        /// 32-bit floating point number.
        F32(u8),

        /// Signed 8-bit integer.
        I8(u8),

        /// Signed normalized 8-bit rational.
        I8Norm(u8),

        /// Signed 16-bit integer.
        I16(u8),

        /// Signed normalized 16-bit rational.
        I16Norm(u8),

        /// Signed 32-bit integer.
        I32(u8),

        /// Unsigned 8-bit integer.
        U8(u8),

        /// Unsigned normalized 8-bit rational.
        U8Norm(u8),

        /// Unsigned 16-bit integer.
        U16(u8),

        /// Unsigned normalized 16-bit rational.
        U16Norm(u8),

        /// Unsigned 32-bit integer.
        U32(u8),
    }

    impl Format {
        /// Returns the corresponding GL data type enumeration constant.
        pub(crate) fn gl_data_type(self) -> u32 {
            match self {
                Format::F32(_) => gl::FLOAT,
                Format::I8(_) | Format::I8Norm(_) => gl::BYTE,
                Format::I16(_) | Format::I16Norm(_) => gl::SHORT,
                Format::I32(_) => gl::INT,
                Format::U8(_) | Format::U8Norm(_) => gl::UNSIGNED_BYTE,
                Format::U16(_) | Format::U16Norm(_) => gl::UNSIGNED_SHORT,
                Format::U32(_) => gl::UNSIGNED_INT,
            }
        }

        /// Returns true if this is a normalized type.
        pub fn norm(self) -> bool {
            match self {
                Format::I8Norm(_) => true,
                Format::I16Norm(_) => true,
                Format::U8Norm(_) => true,
                Format::U16Norm(_) => true,
                _ => false,
            }
        }

        /// Returns the number of elements.
        pub fn size(self) -> usize {
            let size = match self {
                Format::F32(size) => size,
                Format::I8(size) => size,
                Format::I8Norm(size) => size,
                Format::I16(size) => size,
                Format::I16Norm(size) => size,
                Format::I32(size) => size,
                Format::U8(size) => size,
                Format::U8Norm(size) => size,
                Format::U16(size) => size,
                Format::U16Norm(size) => size,
                Format::U32(size) => size,
            };
            match size {
                1 | 2 | 3 | 4 => size as usize,
                _ => panic!("invalid buffer format size"),
            }
        }
    }

    macro_rules! impl_from_format {
        ( $($ident:ident,)* ) => {
            $(
                impl From<$ident> for Format {
                    fn from(item: $ident) -> Format {
                        Format::$ident(item.0)
                    }
                }
            )*
        };
    }

    impl_from_format!(
        F32,
        I8,
        I8Norm,
        I16,
        I16Norm,
        I32,
        U8,
        U8Norm,
        U16,
        U16Norm,
        U32,
    );
}
