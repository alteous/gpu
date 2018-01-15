extern crate vec_map;

mod factory;
mod gl;

pub mod buffer;
pub mod program;
pub mod vertex_array;

#[doc(inline)]
pub use buffer::Buffer;

#[doc(inline)]
pub use factory::Factory;

#[doc(inline)]
pub use program::Program;

#[doc(inline)]
pub use vertex_array::VertexArray;

