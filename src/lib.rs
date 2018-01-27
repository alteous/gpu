extern crate arrayvec;
extern crate crossbeam_channel;
#[macro_use] extern crate log;
extern crate vec_map;

mod factory;
mod queue;
pub mod gl;

pub mod buffer;
pub mod draw_call;
pub mod program;
pub mod texture;
pub mod vertex_array;

/// Fixed size vector type.
pub type ArrayVec<T> = arrayvec::ArrayVec<T>;

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
pub use program::Invocation;

#[doc(inline)]
pub use program::Program;

#[doc(inline)]
pub use texture::Sampler;

#[doc(inline)]
pub use texture::Texture2;

#[doc(inline)]
pub use vertex_array::VertexArray;

