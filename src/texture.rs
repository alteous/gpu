//! Texture2 objects.

use gl;
use queue;
use std::{cmp, fmt, hash, ops, sync};

/// OpenGL texture ID type.
pub type Id = u32;

/// Texture filtering mode.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Filter {
    /// Linear filter.
    Linear,
}

impl Filter {
    pub(crate) fn as_gl_enum(self) -> u32 {
        match self {
            Filter::Linear => gl::LINEAR,
        }
    }
}

/// Texture co-ordinate wrapping mode.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Wrap {
    /// Repeat.
    Repeat,
}

impl Wrap {
    pub(crate) fn as_gl_enum(self) -> u32 {
        match self {
            Wrap::Repeat => gl::REPEAT,
        }
    }
}

/// Returns the texture back to the factory upon destruction.
struct Destructor {
    id: Id,
    tx: queue::Sender<Id>,
}

impl ops::Drop for Destructor {
    fn drop(&mut self) {
        let _ = self.tx.send(self.id);
    }
}

#[derive(Clone)]
pub struct Sampler {
    /// The parent texture ID.
    id: Id,

    /// The texture kind (e.g. GL_TEXTURE_2D).
    ty: u32,

    /// Returns the parent texture back to the factory upon destruction.
    _destructor: sync::Arc<Destructor>,
}

impl Sampler {
    /// Construct a sampler from a 2D texture.
    pub fn from_texture2(texture: Texture2) -> Self {
        Self {
            id: texture.id,
            ty: gl::TEXTURE_2D,
            _destructor: texture._destructor,
        }
    }

    /// Returns the OpenGL ID of the parent texture.
    pub(crate) fn id(&self) -> Id {
        self.id
    }

    /// Returns the OpenGL texture type.
    pub(crate) fn ty(&self) -> u32 {
        self.ty
    }
}

impl cmp::Eq for Sampler {}

impl cmp::PartialEq<Self> for Sampler {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl fmt::Debug for Sampler {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        #[derive(Debug)]
        struct Sampler {
            texture: Id,
            kind: u32,
        }

        Sampler {
            texture: self.id,
            kind: self.ty,
        }.fmt(f)
    }
}

impl hash::Hash for Sampler {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

/// Builder for creating textures.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Builder {
    /// Specifies whether mipmaps should be generated upon creation.
    ///
    /// Default: `true`.
    pub generate_mipmaps: bool,

    /// Specifies the magnification filter.
    ///
    /// Default: `Linear`.
    pub mag_filter: Filter,

    /// Specifies the minification filter.
    ///
    /// Default: `Linear`.
    pub min_filter: Filter,

    /// Specifies the wrapping mode for the S axis.
    ///
    /// Default: `Repeat`.
    pub wrap_s: Wrap,

    /// Specifies the wrapping mode for the T axis.
    ///
    /// Default: `Repeat`.
    pub wrap_t: Wrap,
}

impl Builder {
    /// Constructor.
    pub fn new() -> Self {
        Self {
            generate_mipmaps: true,
            mag_filter: Filter::Linear,
            min_filter: Filter::Linear,
            wrap_s: Wrap::Repeat,
            wrap_t: Wrap::Repeat,
        }
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

/// A 2D texture object.
#[derive(Clone)]
pub struct Texture2 {
    /// The OpenGL texture ID.
    id: Id,

    /// Returns the texture back to the factory upon destruction.
    _destructor: sync::Arc<Destructor>,
}

impl Texture2 {
    pub(crate) fn new(
        id: Id,
        tx: queue::Sender<Id>,
    ) -> Self {
        Texture2 {
            id,
            _destructor: sync::Arc::new(Destructor { id, tx }),
        }
    }
    
    /// Constructs a new texture [`Builder`].
    ///
    /// [`Builder`]: struct.Builder.html
    pub fn builder() -> Builder {
        Builder::new()
    }

    /// Returns the OpenGL texture ID.
    pub fn id(&self) -> Id {
        self.id
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

