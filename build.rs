extern crate gl_generator;

use gl_generator::{Registry, Api, Profile, Fallbacks, StructGenerator};

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let path = std::path::Path::new(&out_dir).join("gl.rs");
    let mut file = std::fs::File::create(path).unwrap();
    Registry::new(Api::Gl, (3, 2), Profile::Core, Fallbacks::All, [])
        .write_bindings(StructGenerator, &mut file)
        .unwrap();
}
