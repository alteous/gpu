//! Shader objects.

use gl;
use queue;
use std::{cmp, ffi, fmt, hash, ops, sync};

use program::Destroyed;

/// Shader source code type.
pub type Source = ffi::CStr;

/// Determines the shader type, e.g. a vertex or fragment shader.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Kind {
    /// Corresponds to `GL_VERTEX_SHADER`.
    Vertex,

    /// Corresponds to `GL_FRAGMENT_SHADER`.
    Fragment,
}

impl Kind {
    /// Returns the equivalent OpenGL shader enumeration constant.
    pub(crate) fn as_gl_enum(self) -> u32 {
        match self {
            Kind::Vertex => gl::VERTEX_SHADER,
            Kind::Fragment => gl::FRAGMENT_SHADER,
        }
    }
}

/// Pushes the shader/program ID onto the factory program queue when
/// destroyed.
#[derive(Clone)]
pub(crate) struct ObjectDestructor {
    id: u32,
    tx: queue::Sender<Destroyed>,
}

impl ops::Drop for ObjectDestructor {
    fn drop(&mut self) {
        let _ = self.tx.send(Destroyed::Object(self.id));
    }
}

/// An unlinked component of a GLSL program, e.g. a compiled
/// vertex or fragment shader.
#[derive(Clone)]
pub struct Object {
    /// The OpenGL shader object ID.
    id: u32,

    /// Determines the shader type, e.g. a vertex or fragment shader.
    kind: Kind,

    /// Returns the object back to the factory upon destruction.
    _destructor: sync::Arc<ObjectDestructor>,
}

impl Object {
    /// Constructor.
    pub(crate) fn new(
        id: u32,
        kind: Kind,
        tx: queue::Sender<Destroyed>,
    ) -> Self {
        Self {
            _destructor: sync::Arc::new(
                ObjectDestructor {
                    id,
                    tx,
                },
            ),
            id,
            kind,
        }
    }

    /// Returns the GLSL object ID.
    pub(crate) fn id(&self) -> u32 {
        self.id
    }
}

impl cmp::Eq for Object {}

impl cmp::PartialEq<Self> for Object {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl fmt::Debug for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        #[derive(Debug)]
        struct Object(u32, Kind);

        Object(self.id, self.kind).fmt(f)
    }
}

impl hash::Hash for Object {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}
