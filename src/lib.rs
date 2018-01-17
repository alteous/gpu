extern crate crossbeam_channel;
extern crate vec_map;

mod factory;
mod queue;
pub mod gl;

pub mod buffer;
pub mod program;
pub mod texture;
pub mod vertex_array;

#[doc(inline)]
pub use buffer::Buffer;

#[doc(inline)]
pub use factory::Factory;

#[doc(inline)]
pub use program::Program;

#[doc(inline)]
pub use texture::Sampler;

#[doc(inline)]
pub use texture::Texture2;

#[doc(inline)]
pub use vertex_array::VertexArray;

