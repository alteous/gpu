//! CPU-visible pixel container.

use gl;

// Pixel data types
// ================
// GL_UNSIGNED_BYTE
// GL_BYTE
// GL_UNSIGNED_SHORT
// GL_SHORT
// GL_UNSIGNED_INT
// GL_INT
// GL_FLOAT
// GL_UNSIGNED_BYTE_3_3_2
// GL_UNSIGNED_BYTE_2_3_3_REV
// GL_UNSIGNED_SHORT_5_6_5
// GL_UNSIGNED_SHORT_5_6_5_REV
// GL_UNSIGNED_SHORT_4_4_4_4
// GL_UNSIGNED_SHORT_4_4_4_4_REV
// GL_UNSIGNED_SHORT_5_5_5_1
// GL_UNSIGNED_SHORT_1_5_5_5_REV
// GL_UNSIGNED_INT_8_8_8_8
// GL_UNSIGNED_INT_8_8_8_8_REV
// GL_UNSIGNED_INT_10_10_10_2
// GL_UNSIGNED_INT_2_10_10_10_REV
//
// Pixel channel orders
// ====================
// GL_RED
// GL_RG
// GL_RGB
// GL_BGR
// GL_RGBA
// GL_BGRA

/// An image pixel format.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Format {
    /// 32-bit floating point format.
    F32(F32),

    /// 8-bit unsigned integer format.
    U8(U8),

    /// 32-bit unsigned integer format.
    U32(U32),
}

impl Format {
    pub(crate) fn as_gl_enums(&self) -> (u32, u32) {
        match *self {
            Format::F32(F32::R) => (gl::FLOAT, gl::RED),
            Format::F32(F32::Rg) => (gl::FLOAT, gl::RG),
            Format::F32(F32::Rgb) => (gl::FLOAT, gl::RGB),
            Format::F32(F32::Bgr) => (gl::FLOAT, gl::BGR),
            Format::F32(F32::Rgba) => (gl::FLOAT, gl::RGBA),
            Format::F32(F32::Bgra) => (gl::FLOAT, gl::BGRA),

            Format::U8(U8::R) => (gl::UNSIGNED_BYTE, gl::RED),
            Format::U8(U8::Rg) => (gl::UNSIGNED_BYTE, gl::RG),
            Format::U8(U8::Rgb) => (gl::UNSIGNED_BYTE, gl::RGB),
            Format::U8(U8::Bgr) => (gl::UNSIGNED_BYTE, gl::BGR),
            Format::U8(U8::Rgba) => (gl::UNSIGNED_BYTE, gl::RGBA),
            Format::U8(U8::Bgra) => (gl::UNSIGNED_BYTE, gl::BGRA),

            Format::U32(U32::R) => (gl::UNSIGNED_INT, gl::RED),
            Format::U32(U32::Rg) => (gl::UNSIGNED_INT, gl::RG),
            Format::U32(U32::Rgb) => (gl::UNSIGNED_INT, gl::RGB),
            Format::U32(U32::Bgr) => (gl::UNSIGNED_INT, gl::BGR),
            Format::U32(U32::Rgba) => (gl::UNSIGNED_INT, gl::RGBA),
            Format::U32(U32::Bgra) => (gl::UNSIGNED_INT, gl::BGRA),
        }
    }
}

impl From<F32> for Format {
    fn from(format: F32) -> Self {
        Format::F32(format)
    }
}

impl From<U8> for Format {
    fn from(format: U8) -> Self {
        Format::U8(format)
    }
}

impl From<U32> for Format {
    fn from(format: U32) -> Self {
        Format::U32(format)
    }
}

/// Pixel format where every channel is an unsigned 8-bit integer.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum U8 {
    /// `[R; R; R; R; R, ...]`
    R,

    /// `[R, G; R, G; R, ...]`
    Rg,

    /// `[R, G, B; R, G, ...]`
    Rgb,

    /// `[B, G, R; B, G, ...]`
    Bgr,

    /// `[R, G, B, A; R, ...]`
    Rgba,

    /// `[B, G, R, A; R, ...]`
    Bgra,
}

/// Pixel format where every channel is an unsigned 32-bit integer.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum U32 {
    /// `[R; R; R; R; R, ...]`
    R,

    /// `[R, G; R, G; R, ...]`
    Rg,

    /// `[R, G, B; R, G, ...]`
    Rgb,

    /// `[B, G, R; B, G, ...]`
    Bgr,

    /// `[R, G, B, A; R, ...]`
    Rgba,

    /// `[B, G, R, A; R, ...]`
    Bgra,
}

/// Pixel format where every channel is a 32-bit floating point number.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum F32 {
    /// `[R; R; R; R; R, ...]`
    R,

    /// `[R, G; R, G; R, ...]`
    Rg,

    /// `[R, G, B; R, G, ...]`
    Rgb,

    /// `[B, G, R; B, G, ...]`
    Bgr,

    /// `[R, G, B, A; R, ...]`
    Rgba,

    /// `[B, G, R, A; R, ...]`
    Bgra,
}
