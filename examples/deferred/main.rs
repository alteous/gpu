extern crate env_logger;
extern crate glutin;
extern crate gpu;
extern crate image;

mod util;

use gpu::buffer as buf;
use gpu::image as img;
use gpu::texture as tex;
use std::{io, ops, sync};

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

#[derive(Clone)]
struct Window(sync::Arc<glutin::GlWindow>);

impl gpu::Context for Window {
    fn query_proc_address(&self, symbol: &str) -> *const () {
        use glutin::GlContext;
        (*self.0).get_proc_address(symbol)
    }

    fn dimensions(&self) -> (u32, u32) {
        (*self.0).get_inner_size().unwrap()
    }
}

impl ops::Deref for Window {
    type Target = glutin::GlWindow;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

const POSITION: buf::Format = buf::Format::F32(3);
const NORMAL: buf::Format = buf::Format::F32(3);

const TRIANGLE_DATA: &'static [Vertex] = &[
    Vertex { position: [ -0.5, -0.5, 0.0 ], normal: [ 0.0, 0.0, 1.0 ] },
    Vertex { position: [ 0.5, -0.5, 0.0 ], normal: [ 0.0, 0.0, 1.0 ] },
    Vertex { position: [ 0.0, 0.5, 0.0 ], normal: [ 0.0, 0.0, 1.0 ] },
];

const CLEAR: gpu::framebuffer::ClearOp = gpu::framebuffer::ClearOp {
    color: gpu::framebuffer::ClearColor::Yes {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    },
    depth: gpu::framebuffer::ClearDepth::Yes {
        z: -1.0,
    },
};

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
    let window = Window(sync::Arc::new(window));
    let (_default_framebuffer, factory) = gpu::init(window.clone());

    let mut vbuf = factory.buffer(buf::Kind::Array, buf::Usage::StaticDraw);
    factory.initialize_buffer(&mut vbuf, TRIANGLE_DATA);

    let positions = buf::Accessor::new(vbuf.clone(), POSITION, 0, 24);
    let normals = buf::Accessor::new(vbuf, NORMAL, 12, 24);
    let attributes = [Some(positions), Some(normals), None, None, None, None, None, None];
    let indices = None;
    let vertex_array = factory.vertex_array(attributes, indices);

    let vertex_shader = {
        let mut source = util::read_file_to_end("examples/deferred/gbuffer.vert").unwrap();
        source.push(0);
        factory.shader(gpu::shader::Kind::Vertex, util::cstr(&source))
    };
    let fragment_shader = {
        let mut source = util::read_file_to_end("examples/deferred/gbuffer.frag").unwrap();
        source.push(0);
        factory.shader(gpu::shader::Kind::Fragment, util::cstr(&source))
    };
    let bindings = gpu::program::Bindings::default();
    let program = factory.program(
        &vertex_shader,
        &fragment_shader,
        &bindings,
    );

    let draw_call = gpu::DrawCall {
        kind: gpu::draw_call::Kind::Arrays,
        primitive: gpu::draw_call::Primitive::Triangles,
        offset: 0,
        count: 3,
    };
    let invocation = gpu::program::Invocation {
        program: &program,
        uniforms: [None; gpu::program::MAX_UNIFORM_BLOCKS],
        samplers: [None; gpu::program::MAX_SAMPLERS],
    };
    let (width, height) = (1920, 1080);
    let format = tex::Format::Rgb32f;
    let position_target = factory.texture2(width, height, false, format);
    let format = tex::Format::Rgb8;
    let normal_target = factory.texture2(width, height, false, format);
    let color_attachments = [
        gpu::framebuffer::ColorAttachment::Texture2(position_target.clone()),
        gpu::framebuffer::ColorAttachment::Texture2(normal_target.clone()),
        gpu::framebuffer::ColorAttachment::None,
    ];
    let framebuffer = factory.framebuffer(width, height, color_attachments);

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

        factory.clear(&framebuffer, CLEAR);
        let state = gpu::pipeline::State::default();
        factory.draw(
            &framebuffer,
            &state,
            &vertex_array,
            &draw_call,
            &invocation,
        );
        window.swap_buffers().unwrap();
    }

    let mut buffer = vec![0u8; 1920 * 1080 * 3];
    factory.read_texture2(
        &normal_target,
        img::U8::Rgb,
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
