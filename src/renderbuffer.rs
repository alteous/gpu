pub(crate) type Id = u32;

/// Buffer optimized as a render target.
#[derive(Debug)]
pub struct Renderbuffer {
    id: Id,
}

impl Renderbuffer {
    /// Constructor.
    pub(crate) fn new(id: Id) -> Self {
        Self { id }
    }

    /// Returns the implicit renderbuffer object.
    pub(crate) fn implicit() -> Self {
        Self { id: 0 }
    }

    /// Returns the OpenGL renderbuffer ID.
    pub(crate) fn id(&self) -> Id {
        self.id
    }
}
