//! Texture plus sampling properties.

use gl;
use std::{cmp, fmt, hash, sync};
use texture;

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

/// Texture plus sampling properties.
#[derive(Clone)]
pub struct Sampler {
    /// The OpenGL texture ID.
    id: texture::Id,

    /// The texture kind, e.g. `TEXTURE_2D`.
    ty: u32,

    /// Returns the texture back to the factory upon destruction.
    _destructor: sync::Arc<texture::Destructor>,

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

impl Sampler {
    /// Construct a sampler from a 2D texture.
    pub fn from_texture2(texture: texture::Texture2) -> Self {
        Self {
            id: texture.id(),
            ty: gl::TEXTURE_2D,
            mag_filter: Filter::Linear,
            min_filter: Filter::Linear,
            wrap_t: Wrap::Repeat,
            wrap_s: Wrap::Repeat,
            _destructor: texture._destructor,
        }
    }

    /// Returns the OpenGL ID of the parent texture.
    pub(crate) fn id(&self) -> texture::Id {
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
            texture: texture::Id,
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
