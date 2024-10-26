use glium::glutin::event::{Event, WindowEvent};
use glium::glutin::event_loop::{ControlFlow, EventLoop};
use glium::glutin::window::WindowBuilder;
use glium::{glutin, Surface};
use glium::{implement_vertex, uniform, Program};
use std::time::Instant;
use structopt::StructOpt;

mod shape_grammar;
use shape_grammar::{BaseShape, Modifier, ShapeGrammar};

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

struct ShaderManager {
    program: Program,
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

        let fragment_shader = include_str!("shaders/fragment.glsl");

        let program = Program::from_source(display, vertex_shader, fragment_shader, None)?;

        Ok(Self {
            program,
            start_time: Instant::now(),
        })
    }

    fn update_uniforms(&self, shape_grammar: &ShapeGrammar) -> impl glium::uniforms::Uniforms {
        let params = &shape_grammar.parameters;
        let time = self.start_time.elapsed().as_secs_f32();

        // Scale the base radius to keep shapes in view
        let base_scale = 0.3 + (params.scale * 0.2);

        // Convert variation arrays into vec4 arrays
        let mut var1_vec4s = [[0.0f32; 4]; 4];
        let mut var2_vec4s = [[0.0f32; 4]; 4];
        let mut var3_vec4s = [[0.0f32; 4]; 4];

        for i in 0..4 {
            for j in 0..4 {
                let idx = i * 4 + j;
                var1_vec4s[i][j] = params.variations[0][idx] * 0.5; // Reduce variation intensity
                var2_vec4s[i][j] = params.variations[1][idx] * 0.3;
                var3_vec4s[i][j] = params.variations[2][idx] * 0.2;
            }
        }

        // Animate light positions
        let light_rotation = time * 0.2;
        let main_light_pos = [
            2.0 * f32::cos(light_rotation),
            2.0,
            2.0 * f32::sin(light_rotation),
        ];

        uniform! {
            // Time and animation
            time: time,
            speed: 0.2f32,

            // Base shape parameters
            base_shape_type: match shape_grammar.base_shape {
                BaseShape::Sphere => 0i32,
                BaseShape::Box => 1i32,
                BaseShape::Cylinder => 2i32,
                BaseShape::Torus => 3i32,
                BaseShape::Cone => 4i32,
                BaseShape::Capsule => 5i32,
                BaseShape::Prism => 6i32,
            },
            base_radius: base_scale,
            base_color: {
                let base = params.base_color;
                let saturation = 1.2;
                [
                    0.3 + base[0] * saturation,
                    0.3 + base[1] * saturation,
                    0.3 + base[2] * saturation,
                ]
            },

            // Transform parameters
            shape_scale: [base_scale, base_scale, base_scale],
            shape_rotation: [
                params.rotation * std::f32::consts::PI / 180.0,
                params.rotation * std::f32::consts::PI / 90.0,
                params.rotation * std::f32::consts::PI / 45.0,
            ],

            // Material properties
            metallic: 0.3f32 + var1_vec4s[0][0] * 0.4,
            roughness: 0.2f32 + var1_vec4s[0][1] * 0.6,

            // Deformation parameters
            deform_factor: var1_vec4s[0][2],
            pattern: (params.shape_type * 4.0) as u32,

            // Variation buffers as vec4 arrays
            var1_0: var1_vec4s[0],
            var1_1: var1_vec4s[1],
            var1_2: var1_vec4s[2],
            var1_3: var1_vec4s[3],

            var2_0: var2_vec4s[0],
            var2_1: var2_vec4s[1],
            var2_2: var2_vec4s[2],
            var2_3: var2_vec4s[3],

            var3_0: var3_vec4s[0],
            var3_1: var3_vec4s[1],
            var3_2: var3_vec4s[2],
            var3_3: var3_vec4s[3],

            // Individual light positions
            light_pos_0: main_light_pos,
            light_pos_1: [-3.0f32, 1.0, 2.0],
            light_pos_2: [0.0f32, 2.0, -3.0],
            light_pos_3: [0.0f32, 4.0, 0.0],

            // Individual light colors
            light_color_0: [1.0f32, 0.9, 0.8],    // Main light (warm)
            light_color_1: [0.4f32, 0.5, 0.6],    // Fill light (cool)
            light_color_2: [0.5f32, 0.5, 0.6],    // Rim light (neutral)
            light_color_3: [0.6f32, 0.6, 0.6],    // Top light (neutral)


            // Modifier parameters
            modifier_count: shape_grammar.modifiers.len() as i32,
            modifier_types: {
                let mut types = [0i32; 4];
                for (i, modifier) in shape_grammar.modifiers.iter().enumerate().take(4) {
                    types[i] = match modifier {
                        Modifier::Twist { .. } => 0i32,
                        Modifier::Bend { .. } => 1i32,
                        Modifier::Round { .. } => 2i32,
                        Modifier::Pattern { .. } => 3i32,
                    };
                }
                types
            },
            modifier_params: {
                let mut params = [0.0f32; 4];
                for (i, modifier) in shape_grammar.modifiers.iter().enumerate().take(4) {
                    params[i] = match modifier {
                        Modifier::Twist { amount } => *amount,
                        Modifier::Bend { amount } => *amount,
                        Modifier::Round { radius } => *radius,
                        Modifier::Pattern { frequency, amplitude } => *frequency * *amplitude,
                    };
                }
                params
            },
        }
    }
}

fn create_display(event_loop: &EventLoop<()>, cli: &Cli) -> glium::Display {
    let window_builder = if cli.fullscreen {
        WindowBuilder::new()
            .with_title("Procedural Shape Generator")
            .with_fullscreen(Some(glutin::window::Fullscreen::Borderless(
                event_loop.primary_monitor(),
            )))
    } else {
        WindowBuilder::new()
            .with_title("Procedural Shape Generator")
            .with_inner_size(glutin::dpi::LogicalSize::new(
                cli.width as f64,
                cli.height as f64,
            ))
    };

    let context_builder = glutin::ContextBuilder::new()
        .with_depth_buffer(24)
        .with_srgb(true)
        .with_stencil_buffer(0)
        .with_vsync(true);

    glium::Display::new(window_builder, context_builder, event_loop)
        .expect("Failed to create display")
}

fn create_buffers(
    display: &glium::Display,
) -> (glium::VertexBuffer<Vertex>, glium::IndexBuffer<u16>) {
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

    let indices = vec![0u16, 1, 2, 2, 3, 0];

    let vertex_buffer =
        glium::VertexBuffer::new(display, &vertices).expect("Failed to create vertex buffer");

    let index_buffer = glium::IndexBuffer::new(
        display,
        glium::index::PrimitiveType::TrianglesList,
        &indices,
    )
    .expect("Failed to create index buffer");

    (vertex_buffer, index_buffer)
}

fn main() {
    let cli = Cli::from_args();

    let event_loop = EventLoop::new();
    let display = create_display(&event_loop, &cli);

    let shader_manager = ShaderManager::new(&display).expect("Failed to create shader manager");

    let shape_grammar = ShapeGrammar::from_hash(&cli.input);
    let (vertex_buffer, index_buffer) = create_buffers(&display);

    let draw_parameters = glium::DrawParameters {
        depth: glium::Depth {
            test: glium::draw_parameters::DepthTest::IfLess,
            write: true,
            ..Default::default()
        },
        backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise,
        ..Default::default()
    };

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
                let mut target = display.draw();
                target.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);

                let uniforms = shader_manager.update_uniforms(&shape_grammar);

                target
                    .draw(
                        &vertex_buffer,
                        &index_buffer,
                        &shader_manager.program,
                        &uniforms,
                        &draw_parameters,
                    )
                    .unwrap();

                target.finish().unwrap();
                display.gl_window().window().request_redraw();
            }
            Event::RedrawEventsCleared => {
                std::thread::sleep(std::time::Duration::from_millis(16));
            }
            _ => (),
        }
    });
}
