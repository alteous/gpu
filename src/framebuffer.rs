use std::{cmp, fmt, hash};

use renderbuffer::Renderbuffer;
use texture::Texture2;

pub const MAX_COLOR_ATTACHMENTS: usize = 3;

pub(crate) type Id = u32;

/// Framebuffer color attachment.
#[derive(Debug)]
pub enum ColorAttachment {
    Renderbuffer(Renderbuffer),

    /// Render to 2D texture.
    Texture2(Texture2),

    /// No color attachment.
    None,
}

/// A framebuffer object.
pub struct Framebuffer {
    /// The OpenGL framebuffer ID.
    id: Id,

    /// Color attachments.
    color_attachments: [ColorAttachment; MAX_COLOR_ATTACHMENTS],
}

impl Framebuffer {
    /// Constructor.
    ///
    /// The caller is responsible for setting up the framebuffer.
    pub(crate) fn new(
        id: Id,
        color_attachments: [ColorAttachment; MAX_COLOR_ATTACHMENTS],
    ) -> Self {
        Self {
            id,
            color_attachments,
        }
    }

    /// Returns the implicit framebuffer object.
    pub(crate) fn implicit() -> Self {
        Self {
            id: 0,
            color_attachments: [
                ColorAttachment::Renderbuffer(Renderbuffer::implicit()),
                ColorAttachment::None,
                ColorAttachment::None,
            ],
        }
    }

    /// Returns the OpenGL framebuffer ID.
    pub(crate) fn id(&self) -> Id {
        self.id
    }
}

impl fmt::Debug for Framebuffer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        #[derive(Debug)]
        struct Framebuffer<'a> {
            id: Id,
            color_attachments: &'a [ColorAttachment],
        }

        Framebuffer {
            id: self.id,
            color_attachments: &self.color_attachments,
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
