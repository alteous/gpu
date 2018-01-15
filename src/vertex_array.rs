//! Vertex array objects.

use buffer::Accessor;
use std::{cmp, fmt, hash, ops};
use std::sync::{self, mpsc};
use vec_map::VecMap;

/// The maximum number of vertex attributes permitted by the crate.
pub const MAX_ATTRIBUTES: usize = 8;

/// The OpenGL VAO ID type.
pub(crate) type Id = u32;

/// Returns the VAO back to the factory upon destruction.
struct Destructor {
    id: u32,
    tx: mpsc::Sender<Id>
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
    indices: Option<Accessor>,

    /// Vertex attributes to bind at draw time.
    attributes: VecMap<Accessor>,

    /// Returns the VAO back to the factory upon destruction.
    destructor: sync::Arc<Destructor>,
}

impl VertexArray {
    /// Constructor.
    pub(crate) fn new(
        id: Id,
        builder: Builder,
        tx: mpsc::Sender<Id>,
    ) -> Self {
        Self {
            id,
            indices: builder.indices,
            attributes: builder.attributes,
            destructor: sync::Arc::new(Destructor { id, tx }),
        }
    }

    /// Returns the OpenGL VAO ID.
    pub(crate) fn id(&self) -> Id {
        self.id
    }

    /// Returns the accessor bound as the element array buffer.
    pub fn indices(&self) -> Option<&Accessor> {
        self.indices.as_ref()
    }

    /// Returns the accessor bound to the given attribute index.
    pub fn attribute(&self, index: u8) -> Option<&Accessor> {
        self.attributes.get(index as usize)
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
            indices: &'a Option<Accessor>,
            attributes: &'a VecMap<Accessor>,
        }

        VertexArray {
            id: self.id,
            indices: &self.indices,
            attributes: &self.attributes,
        }.fmt(f)
    }
}

impl hash::Hash for VertexArray {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}

/// A vertex array object definition.
#[derive(Clone, Debug, Default)]
pub struct Builder {
    /// Draw sequence indices to bind at draw time.
    pub indices: Option<Accessor>,

    /// Vertex attributes to bind at draw time.
    pub attributes: VecMap<Accessor>,
}

impl VertexArray {
    /// Begin building a new vertex array object.
    pub fn builder() -> Builder {
        Builder::new()
    }
}

impl Builder {
    /// Constructor.
    pub fn new() -> Self {
        Self {
            indices: None,
            attributes: VecMap::new(),
        }
    }

    /// Bind an accessor to the given attribute index.
    pub fn attribute(&mut self, id: u8, accessor: Accessor) -> &mut Self {
        self.attributes.insert(id as usize, accessor);
        self
    }

    /// Bind an accessor as the index draw sequence.
    pub fn indices(&mut self, accessor: Accessor) -> &mut Self {
        self.indices = Some(accessor);
        self
    }
}
