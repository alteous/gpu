//! Rendering targets.

use queue;
use std::{cmp, fmt, hash, ops, sync};

use renderbuffer::Renderbuffer;
use texture::Texture2;
use Context;

pub const MAX_COLOR_ATTACHMENTS: usize = 3;

pub(crate) type Id = u32;

/// Framebuffer color attachment.
#[derive(Clone, Debug)]
pub enum ColorAttachment {
    /// Render to renderbuffer.
    Renderbuffer(Renderbuffer),

    /// Render to 2D texture.
    Texture2(Texture2),

    /// No color attachment.
    None,
}

/// Framebuffer depth/stencil attachment.
#[derive(Clone, Debug)]
pub enum DepthStencilAttachment {
    /// Use a 2D texture as the depth attachment.
    DepthOnly(Texture2),

    /// Use a 2D texture as the stencil attachment.
    StencilOnly(Texture2),

    /// Use a 2D texture as both the depth and stencil attachment.
    Both(Texture2),

    /// No depth nor stencil attachment.
    None,
}

/// The framebuffer width and height.
#[derive(Clone)]
pub enum Dimensions {
    /// Framebuffer dimensions are known internally by the crate.
    Internal { width: u32, height: u32 },

    /// Framebuffer dimensions must be queried from outside of the crate.
    External { context: sync::Arc<Context> }
}

struct Destructor {
    id: Id,
    tx: queue::Sender<Id>,
}

impl ops::Drop for Destructor {
    fn drop(&mut self) {
        let _ = self.tx.send(self.id);
    }
}

/// Framebuffer attachments.
#[derive(Clone)]
enum Attachments {
    External,
    Internal {
        color: [ColorAttachment; MAX_COLOR_ATTACHMENTS],
        depth_stencil: DepthStencilAttachment, 
    },
}

/// A framebuffer object.
#[derive(Clone)]
pub struct Framebuffer {
    /// The OpenGL framebuffer ID.
    id: Id,

    /// Sends the framebuffer back to the factory upon destruction.
    destructor: sync::Arc<Destructor>,

    /// The framebuffer width and height.
    dimensions: Dimensions,

    /// Framebuffer attachments.
    attachments: Attachments,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ClearColor {
    Yes { r: f32, g: f32, b: f32, a: f32 },
    No,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ClearDepth {
    Yes { z: f64 },
    No,
}

/// Clear operation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClearOp {
    /// Clear color.
    pub color: ClearColor,

    /// Depth value.
    pub depth: ClearDepth,
}

impl Framebuffer {
    /// Constructor for an internally managed framebuffer object.
    pub(crate) fn internal(
        id: Id,
        tx: queue::Sender<Id>,
        width: u32,
        height: u32,
        color: [ColorAttachment; MAX_COLOR_ATTACHMENTS],
        depth_stencil: DepthStencilAttachment,
    ) -> Self {
        Self {
            id,
            destructor: sync::Arc::new(Destructor { id, tx }),
            dimensions: Dimensions::Internal { width, height },
            attachments: Attachments::Internal { color, depth_stencil },
        }
    }

    /// Constructor for an externally managed framebuffer object.
    pub(crate) fn external(
        context: sync::Arc<Context>,
        tx: queue::Sender<Id>,
    ) -> Self {
        Self {
            id: 0,
            destructor: sync::Arc::new(Destructor { id: 0, tx }),
            dimensions: Dimensions::External { context },
            attachments: Attachments::External,
        }
    }

    /// Returns the OpenGL framebuffer ID.
    pub(crate) fn id(&self) -> Id {
        self.id
    }

    /// Returns the aspect ratio of the rendering region.
    pub fn aspect_ratio(&self) -> f32 {
        let (width, height) = self.dimensions();
        width as f32 / height as f32
    }

    /// Returns the width of the rendering region in pixels.
    pub fn dimensions(&self) -> (u32, u32) {
        match self.dimensions {
            Dimensions::Internal { width, height } => (width, height),
            Dimensions::External { ref context } => context.dimensions(),
        }
    }
}

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

impl cmp::PartialEq<Self> for Framebuffer {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl cmp::Eq for Framebuffer {}

impl hash::Hash for Framebuffer {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
