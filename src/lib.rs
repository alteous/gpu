//! Lean OpenGL 3.2 wrapper library.

extern crate crossbeam_channel;
#[macro_use] extern crate log;
extern crate vec_map;

#[cfg(feature = "macros")]
#[macro_use]
pub mod macros;

mod factory;
mod gl;
mod queue;
mod util;

pub mod buffer;
pub mod draw_call;
pub mod framebuffer;
pub mod image;
pub mod program;
pub mod pipeline;
pub mod renderbuffer;
pub mod sampler;
pub mod shader;
pub mod texture;
pub mod vertex_array;

use std::boxed::Box;

/// Represents an OpenGL context.
pub trait Context {
    /// Retrieve the OpenGL function address for the given symbol.
    fn query_proc_address(&self, symbol: &str) -> *const ();

    /// Retrieve the dimensions of the context's framebuffer object.
    fn dimensions(&self) -> (u32, u32);
}

/// Initialize the library, creating a default framebuffer to render to and
/// a factory to instantiate library objects.
pub fn init<T>(context: T) -> (Framebuffer, Factory)
    where T: Context + 'static
{
    let factory = Factory::new(|symbol| context.query_proc_address(symbol));
    let framebuffer = Framebuffer::external(Box::new(context));
    (framebuffer, factory)
}

#[doc(inline)]
pub use buffer::Accessor;

#[doc(inline)]
pub use buffer::Buffer;

#[doc(inline)]
pub use draw_call::DrawCall;

#[doc(inline)]
pub use factory::Factory;

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
pub use texture::Texture2;

#[doc(inline)]
pub use sampler::Sampler;

#[doc(inline)]
pub use vertex_array::VertexArray;
