//! GLSL programs.

use queue;
use std::{cmp, fmt, hash, ops, sync};

use buffer::Buffer;
use sampler::Sampler;

/// Specifies the maximum number of uniforms permitted by the crate.
pub const MAX_UNIFORM_BLOCKS: usize = 4;

/// Specifies the maximum number of samplers permitted by the crate.
pub const MAX_SAMPLERS: usize = 4;

/// Program interface binding points.
///
/// The binding indices are the array indices items
/// are assigned to.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Bindings {
    /// The program uniform block bindings.
    pub uniform_blocks: [UniformBlockBinding; MAX_UNIFORM_BLOCKS],

    /// The program sampler bindings.
    pub samplers: [SamplerBinding; MAX_SAMPLERS],
}

/// A binding point for a uniform block in a compiled and linked program.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum UniformBlockBinding {
    /// Binding point is required by the program to function correctly.
    Required(&'static [u8]),

    /// Binding point is unassigned.
    None,
}

impl Default for UniformBlockBinding {
    fn default() -> Self {
        UniformBlockBinding::None
    }
}

/// A binding point for a texture sampler in a compiled and linked program.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SamplerBinding {
    /// Binding point is required by the program to function correctly.
    Required(&'static [u8]),

    /// Binding point is unassigned.
    None,
}

impl Default for SamplerBinding {
    fn default() -> Self {
        SamplerBinding::None
    }
}

/// Specifies whether the destroyed item was an object or a program.
#[derive(Clone)]
pub(crate) enum Destroyed {
    /// A shader object.
    Object(u32),

    /// A compiled and linked program.
    Program(u32),
}

/// Pushes the shader/program ID onto the factory program queue when
/// destroyed.
#[derive(Clone)]
pub(crate) struct ProgramDestructor {
    id: u32,
    tx: queue::Sender<Destroyed>,
}

impl ops::Drop for ProgramDestructor {
    fn drop(&mut self) {
        let _ = self.tx.send(Destroyed::Program(self.id));
    }
}

/// An invocation of a shader program.
#[derive(Clone)]
pub struct Invocation<'a> {
    /// The program to bind at draw time.
    pub program: &'a Program,

    /// Uniform buffers to be bound to the program at draw time.
    pub uniforms: [Option<&'a Buffer>; MAX_UNIFORM_BLOCKS],

    /// Texture samplers to be bound to the program at draw time.
    pub samplers: [Option<&'a Sampler>; MAX_SAMPLERS],
}

/// A compiled shader program.
#[derive(Clone)]
pub struct Program {
    /// The OpenGL program ID.
    id: u32,

    /// Locations of samplers.
    pub(crate) samplers: [Option<u32>; MAX_SAMPLERS],

    /// Returns the program back to the factory upon destruction.
    _destructor: sync::Arc<ProgramDestructor>,
}

impl Program {
    /// Constructor.
    pub(crate) fn new(
        id: u32,
        tx: queue::Sender<Destroyed>,
    ) -> Self {
        Self {
            id,
            samplers: [None; MAX_SAMPLERS],
            _destructor: sync::Arc::new(
                ProgramDestructor {
                    id,
                    tx,
                },
            ),
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
