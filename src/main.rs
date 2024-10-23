extern crate glium;

use glium::glutin::event::{Event, WindowEvent};
use glium::glutin::event_loop::{ControlFlow, EventLoop};
use glium::glutin::window::WindowBuilder;
use glium::uniforms::UniformBuffer;
use glium::{glutin, implement_vertex, uniform, Surface};
//use sha2::{Digest, Sha256};
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    /// The input string to derive the shape
    input: String,
}

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
}
implement_vertex!(Vertex, position);

fn create_full_screen_quad() -> (Vec<Vertex>, Vec<u16>) {
    let vertices = vec![
        Vertex {
            position: [-1.0, -1.0],
        },
        Vertex {
            position: [1.0, -1.0],
        },
        Vertex {
            position: [1.0, 1.0],
        },
        Vertex {
            position: [-1.0, 1.0],
        },
    ];

    let indices = vec![0, 1, 2, 2, 3, 0];

    (vertices, indices)
}

fn hash_input(input: &str) -> String {
    //let mut hasher = Sha256::new();
    //hasher.update(input);
    //format!("{:x}", hasher.finalize())

    let hash1 = blake3::hash(input.as_bytes());
    hash1.to_hex().to_string()
}

fn safe_hash_value(hash: &str, start: usize, length: usize, default: u8) -> u8 {
    if start + length <= hash.len() {
        u8::from_str_radix(&hash[start..start + length], 16).unwrap_or(default)
    } else {
        default
    }
}

fn derive_shape_parameters(
    hash: &str,
) -> (f32, f32, f32, [f32; 3], [f32; 16], [f32; 16], [f32; 8]) {
    // Basic parameters with safe defaults
    let shape_type = (safe_hash_value(hash, 0, 2, 128) as f32 / 255.0) * 40.0;
    let scale = 1.0 + (safe_hash_value(hash, 2, 2, 128) as f32 / 255.0);
    let rotation = safe_hash_value(hash, 4, 2, 128) as f32 * 360.0 / 255.0;

    // Colors with safe defaults
    let r = safe_hash_value(hash, 6, 2, 128) as f32 / 255.0;
    let g = safe_hash_value(hash, 8, 2, 128) as f32 / 255.0;
    let b = safe_hash_value(hash, 10, 2, 128) as f32 / 255.0;

    let mut variations1 = [0.0; 16];
    let mut variations2 = [0.0; 16];
    let mut variations3 = [0.0; 8];

    // Fill variations with safe values
    for i in 0..16 {
        let start = 12 + i * 2;
        variations1[i] = safe_hash_value(hash, start, 2, 128) as f32 / 255.0;
    }

    for i in 0..16 {
        let start = 44 + i * 2;
        variations2[i] = safe_hash_value(hash, start, 2, 128) as f32 / 255.0;
    }

    for i in 0..8 {
        let start = 76 + i * 2;
        variations3[i] = safe_hash_value(hash, start, 2, 128) as f32 / 255.0;
    }

    (
        shape_type,
        scale,
        rotation,
        [r, g, b],
        variations1,
        variations2,
        variations3,
    )
}
fn main() {
    let args = Cli::from_args();
    let hash = hash_input(&args.input);

    let (shape_type, scale, rotation, base_color, vars1, vars2, vars3) =
        derive_shape_parameters(&hash);

    let event_loop = EventLoop::new();
    let wb = WindowBuilder::new()
        .with_title("Shape Generator")
        .with_inner_size(glutin::dpi::LogicalSize::new(800.0, 600.0))
        .with_transparent(true);

    let cb = glutin::ContextBuilder::new()
        .with_depth_buffer(24)
        .with_vsync(true);

    let display = glium::Display::new(wb, cb, &event_loop).expect("Failed to create display");

    // Create the uniform buffers for variations
    let variations_buffer1 =
        UniformBuffer::new(&display, vars1).expect("Failed to create variations buffer 1");
    let variations_buffer2 =
        UniformBuffer::new(&display, vars2).expect("Failed to create variations buffer 2");
    let variations_buffer3 =
        UniformBuffer::new(&display, vars3).expect("Failed to create variations buffer 3");

    // Rest of setup...
    let (vertices, indices) = create_full_screen_quad();
    let vertex_buffer =
        glium::VertexBuffer::new(&display, &vertices).expect("Failed to create vertex buffer");
    let index_buffer = glium::IndexBuffer::new(
        &display,
        glium::index::PrimitiveType::TrianglesList,
        &indices,
    )
    .expect("Failed to create index buffer");

    // we need to combine the fragment_shader which is split into init, helpers, shapes and main into a single string
    let fragment_shader = include_str!("fragment_shader_init.glsl").to_string()
        + "\n"
        + include_str!("fragment_shader_helpers.glsl")
        + "\n"
        + include_str!("fragment_shader_shapes.glsl")
        + "\n"
        + include_str!("fragment_shader_main.glsl");

    let program = glium::Program::from_source(
        &display,
        include_str!("vertex_shader.glsl"),
        &fragment_shader,
        None,
    )
    .expect("Failed to create shader program");

    let mut time = 0.0f32;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::MainEventsCleared => {
                time += 0.01;

                let mut target = display.draw();
                target.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);

                let uniforms = uniform! {
                    shape_type: shape_type,
                    scale: scale,
                    rotation: rotation,
                    base_color: base_color,
                    variations_buffer1: &variations_buffer1,
                    variations_buffer2: &variations_buffer2,
                    variations_buffer3: &variations_buffer3,
                    time: time,
                };

                target
                    .draw(
                        &vertex_buffer,
                        &index_buffer,
                        &program,
                        &uniforms,
                        &Default::default(),
                    )
                    .unwrap();

                target.finish().unwrap();

                display.gl_window().window().request_redraw();
            }
            _ => (),
        }
    });
}
