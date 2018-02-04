use std::{ffi, fs, io, path};

pub fn cstr<'a, T>(bytes: &'a T) -> &'a ffi::CStr
    where T: AsRef<[u8]> + ?Sized
{
    ffi::CStr::from_bytes_with_nul(bytes.as_ref()).expect("missing NUL byte")
}

pub fn read_file_to_end<P>(path: P) -> io::Result<Vec<u8>>
    where P: AsRef<path::Path>
{
    use io::Read;
    let file = fs::File::open(path)?;
    let mut reader = io::BufReader::new(file);
    let mut contents = Vec::new();
    let _ = reader.read_to_end(&mut contents)?;
    Ok(contents)
}
