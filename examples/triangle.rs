extern crate glutin;
extern crate graphics;

use std::{ffi, fs, io, path};

use graphics::buffer::Format;

use glutin::{Api, GlContext, GlRequest};
use glutin::Event::WindowEvent;
use glutin::WindowEvent::Closed;

#[repr(C)]
struct Vertex {
    position: [f32; 3],
}

#[repr(C)]
struct UniformBlock {
    color: [f32; 4],
}

const POSITION_FORMAT: Format = Format::Float { size: 3, bits: 32 };

const TRIANGLE_DATA: &'static [Vertex] = &[
    Vertex { position: [ -0.5, -0.5, 0.0 ] },
    Vertex { position: [ 0.5, -0.5, 0.0 ] },
    Vertex { position: [ 0.0, 0.5, 0.0 ] },
];

const YELLOW: &'static [UniformBlock] = &[
    UniformBlock { color: [1.0, 1.0, 0.0, 1.0] },
];

fn read_file_to_end<P>(path: P) -> io::Result<Vec<u8>>
    where P: AsRef<path::Path>
{
    use io::Read;
    let file = fs::File::open(path)?;
    let mut reader = io::BufReader::new(file);
    let mut contents = Vec::new();
    let _ = reader.read_to_end(&mut contents)?;
    Ok(contents)
}

fn main() {
    let mut event_loop = glutin::EventsLoop::new();
    let window_builder = glutin::WindowBuilder::new();
    let context_builder = glutin::ContextBuilder::new()
        .with_gl(GlRequest::Specific(Api::OpenGl, (3, 2)))
        .with_vsync(true)
        .with_multisampling(4);
    let window = glutin::GlWindow::new(
        window_builder,
        context_builder,
        &event_loop,
    ).unwrap();
    unsafe { window.make_current().unwrap() }
    let gl = graphics::init(|sym| window.get_proc_address(sym) as *const _);
    let factory = graphics::Factory::new(gl);

    let vertex_shader = {
        let mut source = read_file_to_end("triangle.vs").unwrap();
        source.push(0);
        let cstr = ffi::CStr::from_bytes_with_nul(&source).unwrap();
        factory.program_object(
            graphics::program::Kind::Vertex,
            cstr,
        )
    };
    let fragment_shader = {
        let mut source = read_file_to_end("triangle.fs").unwrap();
        source.push(0);
        let cstr = ffi::CStr::from_bytes_with_nul(&source).unwrap();
        factory.program_object(
            graphics::program::Kind::Fragment,
            cstr,
        )
    };
    let (program, block_binding) = {
        let prog = factory.program(
            &vertex_shader,
            &fragment_shader,
        );
        let name = ffi::CStr::from_bytes_with_nul(b"UniformBlock\0").unwrap();
        let binding = factory.query_uniform_block_index(&prog, name);
        (prog, binding.unwrap() as usize)
    };

    let vertex_buffer = factory.buffer(graphics::buffer::Kind::Array, graphics::buffer::Usage::StaticDraw);
    factory.init(&vertex_buffer, TRIANGLE_DATA);

    let uniform_buffer = factory.buffer(graphics::buffer::Kind::Uniform, graphics::buffer::Usage::DynamicDraw);
    factory.init(&uniform_buffer, YELLOW);
    
    let position_accessor = graphics::buffer::Accessor::new(vertex_buffer, POSITION_FORMAT, 0, 0);
    let mut vertex_array_builder = graphics::VertexArray::builder();
    vertex_array_builder.attributes.insert(0, position_accessor);
    let vertex_array = factory.vertex_array(vertex_array_builder);

    let mut invocation = graphics::program::Invocation {
        program: &program,
        uniforms: [None, None, None, None],
    };
    invocation.uniforms[block_binding] = Some(uniform_buffer);

    let mut running = true;
    while running {
        window.swap_buffers().unwrap();
        factory.draw(&vertex_array, 0 .. 3, &invocation);
        event_loop.poll_events(|event| {
            match event {
                WindowEvent { event: Closed, .. } => running = false,
                _ => {}
            }
        });
    }
}
