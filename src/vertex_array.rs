//! Vertex array objects.

use buffer;
use queue;
use std::{cmp, fmt, hash, ops, sync};

/// The maximum number of vertex attributes permitted by the crate.
pub const MAX_ATTRIBUTES: usize = 8;

/// The OpenGL VAO ID type.
pub(crate) type Id = u32;

/// Vertex attribute.
pub type Attribute = buffer::Accessor;

/// Index data.
pub type Indices = buffer::Accessor;

/// Returns the VAO back to the factory upon destruction.
struct Destructor {
    id: u32,
    tx: queue::Sender<Id>
}

impl ops::Drop for Destructor {
    fn drop(&mut self) {
        let _ = self.tx.send(self.id);
    }
}

/// Corresponds to an OpenGL vertex array object.
#[derive(Clone)]
pub struct VertexArray {
    /// The OpenGL VAO ID.
    id: Id,
    
    /// Draw sequence indices to bind at draw time.
    indices: Option<Indices>,

    /// Vertex attributes to bind at draw time.
    attributes: [Option<Attribute>; MAX_ATTRIBUTES],

    /// Returns the VAO back to the factory upon destruction.
    destructor: sync::Arc<Destructor>,
}

impl VertexArray {
    /// Constructor.
    pub(crate) fn new(
        id: Id,
        attributes: [Option<Attribute>; MAX_ATTRIBUTES],
        indices: Option<Indices>,
        tx: queue::Sender<Id>,
    ) -> Self {
        Self {
            id,
            indices,
            attributes,
            destructor: sync::Arc::new(Destructor { id, tx }),
        }
    }

    /// Returns the OpenGL VAO ID.
    pub(crate) fn id(&self) -> Id {
        self.id
    }

    /// Returns the accessor bound as the element array buffer.
    pub fn indices(&self) -> Option<&Indices> {
        self.indices.as_ref()
    }

    /// Returns the accessor bound to the given attribute index.
    pub fn attribute(&self, index: u8) -> Option<&Attribute> {
        self.attributes[index as usize].as_ref()
    }
}

impl cmp::Eq for VertexArray {}

impl cmp::PartialEq<Self> for VertexArray {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl fmt::Debug for VertexArray {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        #[derive(Debug)]
        struct VertexArray<'a> {
            id: u32,
            indices: Option<&'a Indices>,
            attributes: &'a [Option<Attribute>],
        }

        VertexArray {
            id: self.id,
            indices: self.indices.as_ref(),
            attributes: &self.attributes,
        }.fmt(f)
    }
}

impl hash::Hash for VertexArray {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}
