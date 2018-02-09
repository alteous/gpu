//! GPU-visible pixel container optimized as a render target.

use queue;
use std::{cmp, fmt, hash, ops, sync};

pub(crate) type Id = u32;

struct Destructor {
    id: Id,
    tx: queue::Sender<Id>,
}

impl ops::Drop for Destructor {
    fn drop(&mut self) {
        let _ = self.tx.send(self.id);
    }
}

/// Buffer optimized as a render target.
#[derive(Clone)]
pub struct Renderbuffer {
    id: Id,
    destructor: sync::Arc<Destructor>,
}

impl Renderbuffer {
    /// Constructor.
    pub(crate) fn new(id: Id, tx: queue::Sender<Id>) -> Self {
        Self {
            id,
            destructor: sync::Arc::new(Destructor { id, tx }),
        }
    }

    /// Returns the implicit renderbuffer object.
    pub(crate) fn implicit(tx: queue::Sender<Id>) -> Self {
        Self {
            id: 0,
            destructor: sync::Arc::new(Destructor { id: 0, tx }),
        }
    }

    /// Returns the OpenGL renderbuffer ID.
    pub(crate) fn id(&self) -> Id {
        self.id
    }
}

impl fmt::Debug for Renderbuffer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        #[derive(Debug)]
        struct Renderbuffer {
            id: Id,
        }

        Renderbuffer { id: self.id }.fmt(f)
    }
}

impl cmp::PartialEq<Self> for Renderbuffer {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl cmp::Eq for Renderbuffer {}

impl hash::Hash for Renderbuffer {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
