extern crate arrayvec;
extern crate crossbeam_channel;
#[macro_use] extern crate log;
extern crate vec_map;

mod factory;
mod queue;
pub mod gl;

pub mod buffer;
pub mod draw_call;
pub mod framebuffer;
pub mod program;
pub mod pipeline;
pub mod renderbuffer;
pub mod texture;
pub mod vertex_array;

use std::os;

/// Fixed size vector type.
pub type ArrayVec<T> = arrayvec::ArrayVec<T>;

/// Initialize the library, creating a default framebuffer to render to and
/// a factory to instantiate library objects.
pub fn init<F>(query_proc_address: F) -> (Framebuffer, Factory)
    where F: FnMut(&str) -> *const os::raw::c_void
{
    let factory = Factory::new(query_proc_address);
    let framebuffer = Framebuffer::implicit();
    (framebuffer, factory)
}

#[doc(inline)]
pub use buffer::Accessor;

#[doc(inline)]
pub use buffer::Buffer;

#[doc(inline)]
pub use draw_call::DrawCall;

#[doc(inline)]
pub use draw_call::Mode;

#[doc(inline)]
pub use draw_call::Primitive;

#[doc(inline)]
pub use factory::Factory;

#[doc(inline)]
pub use framebuffer::ColorAttachment;

#[doc(inline)]
pub use framebuffer::Framebuffer;

#[doc(inline)]
pub use program::Invocation;

#[doc(inline)]
pub use program::Program;

#[doc(inline)]
pub use pipeline::State;

#[doc(inline)]
pub use renderbuffer::Renderbuffer;

#[doc(inline)]
pub use texture::Sampler;

#[doc(inline)]
pub use texture::Texture2;

#[doc(inline)]
pub use vertex_array::VertexArray;
