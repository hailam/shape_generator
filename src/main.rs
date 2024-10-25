extern crate glium;

use glium::glutin::event::{Event, WindowEvent};
use glium::glutin::event_loop::{ControlFlow, EventLoop};
use glium::glutin::window::WindowBuilder;
use glium::{glutin, implement_vertex, uniform, Surface};
use std::time::Instant;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    /// The input string to derive the shape
    input: String,

    #[structopt(long, default_value = "800")]
    width: u32,

    #[structopt(long, default_value = "600")]
    height: u32,

    #[structopt(long)]
    fullscreen: bool,
}

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
}
implement_vertex!(Vertex, position);

#[derive(Copy, Clone)]
struct ShapeParams {
    base_radius: f32,
    color: [f32; 3],
    deform_factor: f32,
    speed: f32,
    pattern: u32,
    metallic: f32,
    roughness: f32,
}

struct ShaderManager {
    program: glium::Program,
    start_time: Instant,
}

impl ShaderManager {
    fn new(display: &glium::Display) -> Result<Self, glium::ProgramCreationError> {
        let vertex_shader = r#"
            #version 140
            in vec2 position;
            out vec2 v_position;
            void main() {
                v_position = position;
                gl_Position = vec4(position, 0.0, 1.0);
            }
        "#;

        let fragment_shader = include_str!("./shaders/fragment.glsl");

        let program = glium::Program::from_source(display, vertex_shader, fragment_shader, None)?;

        Ok(Self {
            program,
            start_time: Instant::now(),
        })
    }
}

fn generate_params(input: &str) -> ShapeParams {
    let hash = blake3::hash(input.as_bytes());
    let bytes = hash.as_bytes();

    ShapeParams {
        base_radius: 0.3 + (bytes[0] as f32 / 255.0) * 0.3,
        deform_factor: (bytes[1] as f32 / 255.0) * 0.3,
        speed: 0.5 + (bytes[2] as f32 / 255.0) * 2.0,
        pattern: (bytes[3] % 4) as u32,
        color: [
            (bytes[4] as f32 / 255.0) * 0.8 + 0.2,
            (bytes[5] as f32 / 255.0) * 0.8 + 0.2,
            (bytes[6] as f32 / 255.0) * 0.8 + 0.2,
        ],
        metallic: (bytes[7] as f32 / 255.0),
        roughness: 0.2 + (bytes[8] as f32 / 255.0) * 0.6,
    }
}

fn create_display(event_loop: &EventLoop<()>, cli: &Cli) -> glium::Display {
    let window_builder = if cli.fullscreen {
        WindowBuilder::new()
            .with_title("Shape Generator")
            .with_fullscreen(Some(glutin::window::Fullscreen::Borderless(
                event_loop.primary_monitor(),
            )))
    } else {
        WindowBuilder::new()
            .with_title("Shape Generator")
            .with_inner_size(glutin::dpi::LogicalSize::new(
                cli.width as f64,
                cli.height as f64,
            ))
    };

    let context_builder = glutin::ContextBuilder::new()
        .with_depth_buffer(24)
        .with_vsync(true);

    glium::Display::new(window_builder, context_builder, event_loop)
        .expect("Failed to create display")
}

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

fn main() {
    let cli = Cli::from_args();
    let params = generate_params(&cli.input);

    let event_loop = EventLoop::new();
    let display = create_display(&event_loop, &cli);
    let shader_manager = ShaderManager::new(&display).expect("Failed to create shader program");

    let (vertices, indices) = create_full_screen_quad();
    let vertex_buffer =
        glium::VertexBuffer::new(&display, &vertices).expect("Failed to create vertex buffer");
    let index_buffer = glium::IndexBuffer::new(
        &display,
        glium::index::PrimitiveType::TrianglesList,
        &indices,
    )
    .expect("Failed to create index buffer");

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
                let time = shader_manager.start_time.elapsed().as_secs_f32();

                let mut target = display.draw();
                target.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);

                let uniforms = uniform! {
                    time: time,
                    base_radius: params.base_radius,
                    base_color: params.color,
                    deform_factor: params.deform_factor,
                    speed: params.speed,
                    pattern: params.pattern,
                    metallic: params.metallic,
                    roughness: params.roughness,
                };

                target
                    .draw(
                        &vertex_buffer,
                        &index_buffer,
                        &shader_manager.program,
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
