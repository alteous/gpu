//! GPU-visible pixel container.

use gl;
use queue;
use std::{cmp, fmt, hash, ops, sync};

/// OpenGL texture ID type.
pub(crate) type Id = u32;

/// Texture format descriptors.
pub mod format {
    /// 32-bit float format.
    #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
    pub enum F32 {
        /// Corresponds to `GL_DEPTH_COMPONENT32F`.
        Depth,

        /// Corresponds to `GL_RGB32F`.
        Rgb,

        /// Corresponds to `GL_RGBA32F`.
        Rgba,
    }

    /// 8-bit fixed format.
    #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
    pub enum U8 {
        /// Corresponds to `GL_RGB8`.
        Rgb,

        /// Correponds to `GL_RGBA8`.
        Rgba,
    }
}

/// Format of texture data.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Format {
    /// 32-bit float.
    F32(format::F32),

    /// 8-bit fixed.
    U8(format::U8),
}

impl Format {
    pub(crate) fn as_gl_enum(&self) -> u32 {
        match *self {
            Format::F32(format::F32::Depth) => gl::DEPTH_COMPONENT32F,
            Format::F32(format::F32::Rgb) => gl::RGB32F,
            Format::F32(format::F32::Rgba) => gl::RGBA32F,

            Format::U8(format::U8::Rgb) => gl::RGB8,
            Format::U8(format::U8::Rgba) => gl::RGBA8,
        }
    }
}

impl From<format::F32> for Format {
    fn from(format: format::F32) -> Self {
        Format::F32(format)
    }
}

impl From<format::U8> for Format {
    fn from(format: format::U8) -> Self {
        Format::U8(format)
    }
}

/// Returns the texture back to the factory upon destruction.
pub(crate) struct Destructor {
    id: Id,
    tx: queue::Sender<Id>,
}

impl ops::Drop for Destructor {
    fn drop(&mut self) {
        let _ = self.tx.send(self.id);
    }
}

/// GPU-visible 2D texture.
#[derive(Clone)]
pub struct Texture2 {
    /// The OpenGL texture ID.
    id: Id,

    width: u32,
    height: u32,
    format: Format,
    mipmap: bool,

    /// Returns the texture back to the factory upon destruction.
    ///
    /// Note: This is cloned by `Sampler`.
    pub(crate) _destructor: sync::Arc<Destructor>,
}

impl Texture2 {
    pub(crate) fn new<F: Into<Format>>(
        id: Id,
        width: u32,
        height: u32,
        mipmap: bool,
        format: F,
        tx: queue::Sender<Id>,
    ) -> Self {
        Texture2 {
            id,
            width,
            height,
            mipmap,
            format: format.into(),
            _destructor: sync::Arc::new(Destructor { id, tx }),
        }
    }

    /// Returns the OpenGL texture ID.
    pub(crate) fn id(&self) -> Id {
        self.id
    }

    /// Returns the internal pixel format.
    pub(crate) fn format(&self) -> Format {
        self.format
    }

    /// Returns the width of the texture in pixels.
    pub fn width(&self) -> usize {
        self.width as _
    }

    /// Returns the height of the texture in pixels.
    pub fn height(&self) -> usize {
        self.height as _
    }

    /// Returns `true` if this texture has mipmaps.
    pub fn mipmap(&self) -> bool {
        self.mipmap
    }
}

impl cmp::Eq for Texture2 {}

impl cmp::PartialEq<Self> for Texture2 {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl fmt::Debug for Texture2 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        #[derive(Debug)]
        struct Texture2(u32);

        Texture2(self.id).fmt(f)
    }
}

impl hash::Hash for Texture2 {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

