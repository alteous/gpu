//! Texture plus sampling properties.

use gl;

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

/// Sampling properties for a 2D texture.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Sampler2 {
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

impl Default for Sampler2 {
    fn default() -> Self {
        Self {
            mag_filter: Filter::Linear,
            min_filter: Filter::Linear,
            wrap_s: Wrap::Repeat,
            wrap_t: Wrap::Repeat,
        }
    }
}
