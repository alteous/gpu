//! Helper macros.

/// Returns the offset of a field in a `struct`.
///
/// ```rust
/// # #[macro_use] extern crate gpu;
/// #[repr(C)]
/// struct Vertex {
///     position: [f32; 3],
///     normal: [f32; 3],
/// }
///
/// # fn main() {
/// assert_eq!(0, offset_of!(Vertex::position));
/// assert_eq!(12, offset_of!(Vertex::normal));
/// # }
/// ```
#[macro_export]
macro_rules! offset_of {
    ($ty:ident::$field:ident) => {
        {
            let zero = 0 as *const $ty;
            let offset = unsafe {
                &(*zero).$field as *const _
            };
            offset as usize
        }
    };

    ($ty:ident::$field:ident[$index:expr]) => {
        {
            let zero = 0 as *const $ty;
            let offset = unsafe { 
                &(*zero).$field[$index] as *const _
            };
            offset as usize
        }
    };
}
