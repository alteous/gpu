extern crate glutin;
extern crate graphics;

use std::{fs, io, path};

use graphics::buffer::Format;

use glutin::{Api, GlContext, GlRequest};
use glutin::Event::WindowEvent;
use glutin::WindowEvent::Closed;

#[repr(C)]
struct Vertex {
    position: [f32; 3],
}

const POSITION_FORMAT: Format = Format::Float { size: 3, bits: 32 };

const TRIANGLE_DATA: &'static [Vertex] = &[
    Vertex { position: [ -0.5, -0.5, 0.0 ] },
    Vertex { position: [ 0.5, -0.5, 0.0 ] },
    Vertex { position: [ 0.0, 0.5, 0.0 ] },
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
        factory.program_object(graphics::program::Kind::Vertex, source.as_ptr() as *const _)
    };
    let fragment_shader = {
        let mut source = read_file_to_end("triangle.fs").unwrap();
        source.push(0);
        factory.program_object(graphics::program::Kind::Fragment, source.as_ptr() as *const _)
    };
    let program = factory.program(&*vertex_shader, &*fragment_shader);

    let buffer = factory.buffer(graphics::buffer::Ty::Array, graphics::buffer::Usage::Static);
    factory.init(&buffer, TRIANGLE_DATA);

    let position_accessor = graphics::buffer::Accessor::new(buffer, POSITION_FORMAT, 0, 0);
    let mut vertex_array_builder = graphics::VertexArray::builder();
    vertex_array_builder.attributes.insert(0, position_accessor);
    let vertex_array = factory.vertex_array(vertex_array_builder);

    let draw_call = factory.draw_call(vertex_array.clone(), 0 .. 3, program.clone());

    let mut running = true;
    while running {
        window.swap_buffers().unwrap();
        factory.draw(&draw_call);
        event_loop.poll_events(|event| {
            match event {
                WindowEvent { event: Closed, .. } => running = false,
                _ => {}
            }
        });
    }
}
