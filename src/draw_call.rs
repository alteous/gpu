//! Draw call dispatch.

use gl;

/// Primitive topology.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Primitive {
    /// List of triangle points.
    Triangles,

    /// List of line segments.
    Lines,
}

impl Primitive {
    pub(crate) fn as_gl_enum(self) -> u32 {
        match self {
            Primitive::Triangles => gl::TRIANGLES,
            Primitive::Lines => gl::LINES,
        }
    }
}

/// Draw call kind.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Kind {
    /// Draw arrays once.
    Arrays,

    /// Draw arrays many times.
    ArraysInstanced(usize),

    /// Draw elements once.
    Elements,

    /// Draw elements many times.
    ElementsInstanced(usize),
}

/// A draw call command.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DrawCall {
    /// Where the vertices/elements begin.
    pub offset: usize,

    /// Number of vertices/elements to draw.
    pub count: usize,

    /// The primitive topology.
    pub primitive: Primitive,

    /// Draw call kind.
    pub kind: Kind,
}
