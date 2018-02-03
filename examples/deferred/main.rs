extern crate env_logger;
extern crate glutin;
extern crate gpu;
extern crate image;

use gpu::{buffer as buf, gl};
use std::{ffi, fs, io, path};

use gpu::buffer::Format;
use gpu::texture::PixelFormat;
use glutin::ElementState::Released;
use glutin::Event;
use glutin::GlContext;
use glutin::VirtualKeyCode as Key;
use glutin::WindowEvent;

/// Size: 24
#[repr(C)]
struct Vertex {
    /// Offset: 0
    position: [f32; 3],
    /// Offset: 12
    normal: [f32; 3],
}

const POSITION: Format = Format::Float { size: 3, bits: 32 };
const NORMAL: Format = Format::Float { size: 3, bits: 32 };

const TRIANGLE_DATA: &'static [Vertex] = &[
    Vertex { position: [ -0.5, -0.5, 0.0 ], normal: [ 0.0, 0.0, 1.0 ] },
    Vertex { position: [ 0.5, -0.5, 0.0 ], normal: [ 0.0, 0.0, 1.0 ] },
    Vertex { position: [ 0.0, 0.5, 0.0 ], normal: [ 0.0, 0.0, 1.0 ] },
];

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
    let (framebuffer, factory) = gpu::init(|sym| {
        window.get_proc_address(sym) as *const _
    });

    let vbuf = factory.buffer(buf::Kind::Array, buf::Usage::StaticDraw);
    factory.initialize_buffer(&vbuf, TRIANGLE_DATA);

    let position_accessor= buf::Accessor::new(vbuf.clone(), POSITION, 0, 24);
    let normal_accessor = buf::Accessor::new(vbuf, NORMAL, 12, 24);
    let mut vertex_array_builder = gpu::VertexArray::builder();
    vertex_array_builder.attributes.insert(0, position_accessor);
    vertex_array_builder.attributes.insert(1, normal_accessor);
    let vertex_array = factory.vertex_array(vertex_array_builder);

        let vertex_shader = {
        let mut source = read_file_to_end("examples/deferred/gbuffer.vert").unwrap();
        source.push(0);
        factory.program_object(
            gpu::program::Kind::Vertex,
            cstr(&source),
        )
    };
    let fragment_shader = {
        let mut source = read_file_to_end("examples/deferred/gbuffer.frag").unwrap();
        source.push(0);
        factory.program_object(
            gpu::program::Kind::Fragment,
            cstr(&source),
        )
    };
    let program = factory.program(&vertex_shader, &fragment_shader);

    let draw_call = gpu::DrawCall {
        mode: gpu::Mode::Arrays,
        primitive: gpu::Primitive::Triangles,
        offset: 0,
        count: 3,
    };
    let invocation = gpu::program::Invocation {
        program: &program,
        uniforms: gpu::ArrayVec::new(),
        samplers: gpu::ArrayVec::new(),
    };
    let (width, height) = (1920, 1080);
    let format = PixelFormat::Rgb32f;
    let position_target = factory.texture2(width, height, format);
    let format = PixelFormat::Rgb8;
    let normal_target = factory.texture2(width, height, format);
    let color_attachments = [
        gpu::framebuffer::ColorAttachment::Texture2(position_target.clone()),
        gpu::framebuffer::ColorAttachment::Texture2(normal_target.clone()),
        gpu::framebuffer::ColorAttachment::None,
    ];
    let framebuffer = factory.framebuffer(color_attachments);

    let mut running = true;
    while running {
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
        
        let state = gpu::pipeline::State::default();
        factory.draw(
            &framebuffer,
            &state,
            &vertex_array,
            &draw_call,
            &invocation,
        );
        window.swap_buffers().unwrap();

        println!("break");
        break;
    }

    let mut buffer = vec![0u8; 1920 * 1080 * 3];
    factory.read_texture2(
        &normal_target,
        (gl::RGB, gl::UNSIGNED_BYTE),
        &mut buffer,
    );
    image::save_buffer(
        "normal.png",
        &buffer,
        1920,
        1080,
        image::ColorType::RGB(8),
    ).expect("I/O error");
}
