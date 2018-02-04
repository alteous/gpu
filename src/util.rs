//! Utility functions.

use std::ffi;

/// Safely cast a byte slice into a C string.
pub fn cstr<'a, T>(bytes: &'a T) -> &'a ffi::CStr
    where T: AsRef<[u8]> + ?Sized
{
    ffi::CStr::from_bytes_with_nul(bytes.as_ref()).expect("missing NUL byte")
}
