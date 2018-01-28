use std::{cmp, fmt, hash};

pub(crate) type Id = u32;

/// A framebuffer object.
pub struct Framebuffer {
    /// The OpenGL framebuffer ID.
    id: Id,
}

impl Framebuffer {
    /// Constructor.
    pub(crate) fn new(id: Id) -> Self {
        if id == 0 {
            Framebuffer {
                id,
            }
        } else {
            unimplemented!()
        }
    }

    /// Returns the OpenGL framebuffer ID.
    pub(crate) fn id(&self) -> Id {
        self.id
    }
}

impl Default for Framebuffer {
    fn default() -> Self {
        Framebuffer {
            id: 0,
        }
    }
}

impl cmp::PartialEq<Self> for Framebuffer {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl cmp::Eq for Framebuffer {}

impl fmt::Debug for Framebuffer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        #[derive(Debug)]
        struct Framebuffer {
            id: Id,
        }

        Framebuffer {
            id: self.id,
        }.fmt(f)
    }
}

impl hash::Hash for Framebuffer {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
