//! GLSL programs.

use gl;
use std::{cmp, ffi, fmt, hash, ops};
use std::sync::{self, mpsc};

use buffer::Buffer;
use texture::Sampler;

/// Specifies the maximum number of uniforms permitted by the crate.
pub const MAX_UNIFORMS: usize = 4;

/// Specifies the maximum number of samplers permitted by the crate.
pub const MAX_SAMPLERS: usize = 4;

/// The program source code type.
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

/// Specifies whether the destroyed item was an object or a program.
pub(crate) enum Destroyed {
    /// A shader object.
    Object(u32),

    /// A compiled and linked program.
    Program(u32),
}

/// Pushes the shader/program ID onto the factory program queue when
/// destroyed.
pub(crate) struct ObjectDestructor {
    id: u32,
    tx: mpsc::Sender<Destroyed>,
}

impl ops::Drop for ObjectDestructor {
    fn drop(&mut self) {
        let _ = self.tx.send(Destroyed::Object(self.id));
    }
    
}

/// Pushes the shader/program ID onto the factory program queue when
/// destroyed.
pub(crate) struct ProgramDestructor {
    id: u32,
    tx: mpsc::Sender<Destroyed>,
}

impl ops::Drop for ProgramDestructor {
    fn drop(&mut self) {
        let _ = self.tx.send(Destroyed::Program(self.id));
    }
}

/// An unlinked component of a GLSL program, e.g. a compiled
/// vertex or fragment shader.
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
        tx: mpsc::Sender<Destroyed>,
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

/// An invocation of a shader program.
pub struct Invocation<'a> {
    /// The program to bind at draw time.
    pub program: &'a Program,

    /// Uniform buffers to be bound to the program at draw time.
    pub uniforms: [Option<Buffer>; MAX_UNIFORMS],

    /// Texture samplers to be bound to the program at draw time.
    pub samplers: [Option<Sampler>; MAX_SAMPLERS],
}

/// A compiled shader program.
pub struct Program {
    /// The OpenGL program ID.
    id: u32,

    /// Returns the program back to the factory upon destruction.
    _destructor: sync::Arc<ProgramDestructor>,
}

impl Program {
    /// Constructor.
    pub(crate) fn new(
        id: u32,
        tx: mpsc::Sender<Destroyed>,
    ) -> Self {
        Self {
            _destructor: sync::Arc::new(
                ProgramDestructor {
                    id,
                    tx,
                },
            ),
            id,
        }
    }

    /// Returns the GLSL program ID.
    pub(crate) fn id(&self) -> u32 {
        self.id
    }
}

impl cmp::Eq for Program {}

impl cmp::PartialEq<Self> for Program {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl fmt::Debug for Program {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        #[derive(Debug)]
        struct Program(u32);

        Program(self.id).fmt(f)
    }
}

impl hash::Hash for Program {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}
