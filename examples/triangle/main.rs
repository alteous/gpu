extern crate env_logger;
extern crate glutin;
extern crate gpu;

mod util;

use gpu::{buffer as buf, image as img, texture as tex};
use std::{io, ops, sync};

use glutin::ElementState::Released;
use glutin::Event;
use glutin::GlContext;
use glutin::VirtualKeyCode as Key;
use glutin::WindowEvent;
use gpu::program::{Bindings, SamplerBinding, UniformBlockBinding};

#[repr(C)]
struct Vertex {
    position: [f32; 3],
}

#[repr(C)]
struct UniformBlock {
    color: [f32; 4],
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

const BINDINGS: Bindings = Bindings {
    uniform_blocks: [
        UniformBlockBinding::Required(b"b_Locals\0"),
        UniformBlockBinding::None,
        UniformBlockBinding::None,
        UniformBlockBinding::None,
    ],
    samplers: [
        SamplerBinding::Required(b"u_Sampler\0"),
        SamplerBinding::None,
        SamplerBinding::None,
        SamplerBinding::None,
    ],   
};

const TRIANGLE_VERTICES: &'static [Vertex] = &[
    Vertex { position: [ -0.5, -0.5, 0.0 ] },
    Vertex { position: [ 0.5, -0.5, 0.0 ] },
    Vertex { position: [ 0.0, 0.5, 0.0 ] },
];

const YELLOW: &'static [UniformBlock] = &[
    UniformBlock { color: [1.0, 1.0, 1.0, 1.0] },
];

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
    let (fbuf, factory) = gpu::init(window.clone());

    let vert_shader = {
        let mut source = util::read_file_to_end("examples/triangle/shader.vert")
            .expect("I/O error");
        source.push(0);
        factory.shader(gpu::shader::Kind::Vertex, util::cstr(&source))
    };
    let frag_shader = {
        let mut source = util::read_file_to_end("examples/triangle/shader.frag")
            .expect("I/O error");
        source.push(0);
        factory.shader(gpu::shader::Kind::Fragment, util::cstr(&source))
    };
    let program = factory.program(&vert_shader, &frag_shader, &BINDINGS);
    let mut ubuf = factory.empty_buffer(buf::Kind::Uniform, buf::Usage::DynamicDraw);
    factory.initialize_buffer(&mut ubuf, YELLOW);

    let mut vbuf = factory.empty_buffer(buf::Kind::Array, buf::Usage::StaticDraw);
    factory.initialize_buffer(&mut vbuf, TRIANGLE_VERTICES);
    let positions = buf::Accessor::new(vbuf, buf::format::F32(3), 0, 0);
    let attributes = [Some(positions), None, None, None, None, None, None, None];
    let indices = None;
    let vertex_array = factory.vertex_array(attributes, indices);

    let tex = factory.texture2(1, 1, true, tex::format::U8::Rgba);
    factory.write_texture2(&tex, img::format::U8::Rgba, &[255, 255, 0, 0]);
    let sampler = gpu::Sampler2::default();

    let state = gpu::pipeline::State::default();
    let draw_call = gpu::DrawCall {
        kind: gpu::draw_call::Kind::Arrays,
        primitive: gpu::draw_call::Primitive::Triangles,
        offset: 0,
        count: 3,
    };
    let invocation = gpu::program::Invocation {
        program: &program,
        uniforms: [Some(&ubuf), None, None, None],
        samplers: [Some((&tex, sampler)), None, None, None],
    };

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
        factory.draw(&fbuf, &state, &vertex_array, &draw_call, &invocation);
        window.swap_buffers().unwrap();
    }
}
