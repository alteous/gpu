//! Draw call dispatch.

use gl;

/// Primitive topology.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Primitive {
    /// `[(v0, v1, v2), (v0, v1, v2), ...]`.
    Triangles,

    /// * For odd n, vertices n, n + 1, and n + 2 define triangle n.
    /// * For even n, vertices n + 1, n, and n + 2 define triangle n.
    /// * In total, n - 2 triangles are drawn.
    TriangleStrip,

    /// `[(start, end), (start, end), ...]`.
    Lines,

    /// `[start, end/start, ..., end/start, end]`.
    LineStrip,

    /// `[start, end/start, ..., end/start, end; <implicit_start>]`.
    LineLoop,
}

impl Primitive {
    pub(crate) fn as_gl_enum(self) -> u32 {
        match self {
            Primitive::Triangles => gl::TRIANGLES,
            Primitive::TriangleStrip => gl::TRIANGLE_STRIP,
            Primitive::Lines => gl::LINES,
            Primitive::LineStrip => gl::LINE_STRIP,
            Primitive::LineLoop => gl::LINE_LOOP,
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
