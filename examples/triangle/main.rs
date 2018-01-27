extern crate env_logger;
extern crate glutin;
extern crate gpu;

use gpu::gl;
use std::{ffi, fs, io, path};

use gpu::buffer::Format;

use glutin::ElementState::Released;
use glutin::Event;
use glutin::GlContext;
use glutin::VirtualKeyCode as Key;
use glutin::WindowEvent;

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
    UniformBlock { color: [1.0, 1.0, 1.0, 1.0] },
];

const GREEN_PIXEL: &'static [u32] = &[0x00FFFF];

fn cstr<'a, T>(bytes: &'a T) -> &'a ffi::CStr
    where T: AsRef<[u8]>
{
    ffi::CStr::from_bytes_with_nul(bytes.as_ref()).expect("missing NUL byte")
}

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
    let _ = env_logger::init();
    
    let mut event_loop = glutin::EventsLoop::new();
    let window_builder = glutin::WindowBuilder::new();
    let context_builder = glutin::ContextBuilder::new()
        .with_gl(glutin::GL_CORE)
        .with_vsync(true)
        .with_multisampling(4);
    let window = glutin::GlWindow::new(
        window_builder,
        context_builder,
        &event_loop,
    ).unwrap();
    unsafe { window.make_current().unwrap() }
    let factory = gpu::Factory::new(|sym| {
        window.get_proc_address(sym) as *const _
    });

    let vertex_shader = {
        let mut source = read_file_to_end("examples/triangle/shader.vert").unwrap();
        source.push(0);
        factory.program_object(
            gpu::program::Kind::Vertex,
            cstr(&source),
        )
    };
    let fragment_shader = {
        let mut source = read_file_to_end("examples/triangle/shader.frag").unwrap();
        source.push(0);
        factory.program_object(
            gpu::program::Kind::Fragment,
            cstr(&source),
        )
    };
    let (program, block_binding, sampler_binding) = {
        let prog = factory.program(
            &vertex_shader,
            &fragment_shader,
        );
        let bname = cstr(b"UniformBlock\0");
        let bbinding = factory.query_uniform_block_index(&prog, bname);
        let sname = cstr(b"u_Sampler\0");
        let sbinding = factory.query_uniform_index(&prog, sname);
        (prog, bbinding.unwrap(), sbinding.unwrap())
    };

    let vertex_buffer = factory.buffer(gpu::buffer::Kind::Array, gpu::buffer::Usage::StaticDraw);
    factory.initialize_buffer(&vertex_buffer, TRIANGLE_DATA);

    let uniform_buffer = factory.buffer(gpu::buffer::Kind::Uniform, gpu::buffer::Usage::DynamicDraw);
    factory.initialize_buffer(&uniform_buffer, YELLOW);

    let position_accessor = gpu::buffer::Accessor::new(vertex_buffer, POSITION_FORMAT, 0, 0);
    let mut vertex_array_builder = gpu::VertexArray::builder();
    vertex_array_builder.attributes.insert(0, position_accessor);
    let vertex_array = factory.vertex_array(vertex_array_builder);

    let texture = factory.texture2(Default::default());
    factory.initialize_texture2(
        &texture,
        true,
        gl::RGBA8,
        1,
        1,
        gl::BGRA,
        gl::UNSIGNED_INT_8_8_8_8_REV,
        GREEN_PIXEL,
    );
    let sampler = gpu::Sampler::from_texture2(texture);

    let draw_call = gpu::DrawCall {
        mode: gpu::Mode::Arrays,
        primitive: gpu::Primitive::Triangles,
        offset: 0,
        count: 3,
    };
    let mut invocation = gpu::program::Invocation {
        program: &program,
        uniforms: gpu::ArrayVec::new(),
        samplers: gpu::ArrayVec::new(),
    };
    invocation.uniforms.push((block_binding, &uniform_buffer));
    invocation.samplers.push((sampler_binding, &sampler));

    let mut running = true;
    while running {
        window.swap_buffers().unwrap();
        factory.draw(&vertex_array, &draw_call, &invocation);
        event_loop.poll_events(|event| {
            match event {
                Event::WindowEvent { event, .. } => {
                    match event {
                        WindowEvent::Closed => running = false,
                        WindowEvent::KeyboardInput { input, .. } => {
                            match (input.virtual_keycode, input.state) {
                                (Some(Key::Escape), Released) => {
                                    running = false;
                                },
                                _ => {},
                            }
                        },
                        _ => {}
                    }
                },
                _ => {}
            }
        });
    }
}
