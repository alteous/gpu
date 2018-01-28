use gl;

/// Specifies depth buffer testing.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DepthTest {
    /// Depth test never passes.
    Never,

    /// Depth test passes if the incoming depth value is less than
    /// the stored depth value.
    LessThan,

    /// Depth test always passes.
    Always,
}

impl Default for DepthTest {
    fn default() -> Self {
        DepthTest::LessThan
    }
}

impl DepthTest {
    pub(crate) fn as_gl_enum(&self) -> u32 {
        match *self {
            DepthTest::Never => gl::NEVER,
            DepthTest::LessThan => gl::LESS,
            DepthTest::Always => gl::ALWAYS,
        }
    }
}

/// Specifies the winding order of front facing triangles.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum FrontFace {
    /// Front-facing triangles are clockwise wound.
    Clockwise,

    /// Back-facing triangles are clockwise wound.
    CounterClockwise,
}

impl Default for FrontFace {
    fn default() -> Self {
        FrontFace::CounterClockwise
    }
}

impl FrontFace {
    pub(crate) fn as_gl_enum(&self) -> u32 {
        match *self {
            FrontFace::Clockwise => gl::CW,
            FrontFace::CounterClockwise => gl::CCW,
        }
    }
}

/// Hardware culling mode.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Culling {
    /// Disable culling.
    None,

    /// Cull the front-facing triangles.
    Front,

    /// Cull the back-facing triangles.
    Back,

    /// Cull all faces.
    All,
}

impl Default for Culling {
    fn default() -> Self {
        Culling::Back
    }
}

impl Culling {
    pub(crate) fn as_gl_enum_if_enabled(&self) -> Option<u32> {
        match *self {
            Culling::None => None,
            Culling::Front => Some(gl::FRONT),
            Culling::Back => Some(gl::BACK),
            Culling::All => Some(gl::FRONT_AND_BACK),
        }
    }
}

/// Fixed-function state parameters.
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct State {
    /// Front face winding order.
    pub front_face: FrontFace,

    /// Hardware face culling mode.
    pub culling: Culling,

    /// Hardware depth testing mode.
    pub depth_test: DepthTest,
}
