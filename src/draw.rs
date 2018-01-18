//! Draw call dispatch.

use gl;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Primitive {
    Triangles,
}

impl Primitive {
    pub(crate) fn as_gl_enum(self) -> u32 {
        match self {
            Primitive::Triangles => gl::TRIANGLES,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Mode {
    Arrays,
    ArraysInstanced(usize),
    Elements,
    ElementsInstanced(usize),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Call {
    pub offset: usize,
    pub count: usize,
    pub primitive: Primitive,
    pub mode: Mode,
}
