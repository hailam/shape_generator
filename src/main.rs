mod chain_code;

use chain_code::ChainCodeGenerator;
use glium::{glutin, uniform, Surface};
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(
    name = "shape-generator",
    about = "Generate 3D shapes from input string"
)]
struct Opt {
    /// Input string to generate shape from
    #[structopt(name = "INPUT")]
    input: String,

    /// Grid size for the chain code (default: 0.1)
    #[structopt(long, default_value = "0.1")]
    grid_size: f32,

    /// Window width
    #[structopt(long, default_value = "800")]
    width: u32,

    /// Window height
    #[structopt(long, default_value = "600")]
    height: u32,
}

fn main() {
    let opt = Opt::from_args();

    // Hash the input string
    let hash = blake3::hash(opt.input.as_bytes());
    let bytes = hash.as_bytes();

    // Create window and context
    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_inner_size(glutin::dpi::LogicalSize::new(
            opt.width as f64,
            opt.height as f64,
        ))
        .with_title("Shape Generator");

    let cb = glutin::ContextBuilder::new()
        .with_depth_buffer(24)
        .with_vsync(true);

    let display = glium::Display::new(wb, cb, &event_loop).expect("Failed to create display");

    // Initialize generator
    let bounds = ([-0.8, -0.8, -0.8], [0.8, 0.8, 0.8]);
    let mut generator = ChainCodeGenerator::new(opt.grid_size, bounds);

    // Generate geometry
    let (vertices, indices) = generator.generate(bytes);

    println!(
        "Generated {} vertices and {} indices",
        vertices.len(),
        indices.len()
    );

    // Debug print first few vertices
    println!("First 5 vertices:");
    for i in 0..5.min(vertices.len()) {
        println!(
            "Vertex {}: pos={:?}, normal={:?}, color={:?}",
            i, vertices[i].position, vertices[i].normal, vertices[i].color
        );
    }

    println!("First 15 indices:");
    for i in 0..15.min(indices.len()) {
        print!("{} ", indices[i]);
        if (i + 1) % 3 == 0 {
            println!();
        }
    }

    // Create vertex and index buffers
    let vertex_buffer =
        glium::VertexBuffer::new(&display, &vertices).expect("Failed to create vertex buffer");
    let index_buffer = glium::IndexBuffer::new(
        &display,
        glium::index::PrimitiveType::TrianglesList,
        &indices,
    )
    .expect("Failed to create index buffer");

    // Create shader program
    let program = glium::Program::from_source(
        &display,
        include_str!("shaders/vertex.glsl"),
        include_str!("shaders/fragment.glsl"),
        None,
    )
    .expect("Failed to create shader program");

    let mut time = 0.0f32;

    event_loop.run(move |event, _, control_flow| {
        match event {
            glutin::event::Event::WindowEvent { event, .. } => match event {
                glutin::event::WindowEvent::CloseRequested => {
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                    return;
                }
                _ => return,
            },
            glutin::event::Event::MainEventsCleared => {
                // Update camera
                let camera_angle = time * 0.5;
                let camera_pos = [2.5 * camera_angle.cos(), 1.2, 2.5 * camera_angle.sin()];

                println!("Camera angle: {:.2}, pos: {:?}", camera_angle, camera_pos);

                let view = view_matrix(
                    camera_pos,
                    [0.0, 0.0, 0.0], // Look at center
                    [0.0, 1.0, 0.0], // Up vector
                );

                let perspective = {
                    let (width, height) = display.get_framebuffer_dimensions();
                    let aspect_ratio = width as f32 / height as f32;
                    let fov: f32 = std::f32::consts::PI / 3.0;
                    let znear = 0.1;
                    let zfar = 10.0;

                    [
                        [1.0 / (aspect_ratio * (fov / 2.0).tan()), 0.0, 0.0, 0.0],
                        [0.0, 1.0 / (fov / 2.0).tan(), 0.0, 0.0],
                        [0.0, 0.0, (zfar + znear) / (zfar - znear), 1.0],
                        [0.0, 0.0, -(2.0 * zfar * znear) / (zfar - znear), 0.0],
                    ]
                };

                let matrix = multiply_matrix(perspective, view);

                // Draw frame
                let mut target = display.draw();
                target.clear_color_and_depth((0.1, 0.1, 0.1, 1.0), 1.0);

                let params = glium::DrawParameters {
                    depth: glium::Depth {
                        test: glium::draw_parameters::DepthTest::IfLess,
                        write: true,
                        ..Default::default()
                    },
                    backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise,
                    ..Default::default()
                };

                target
                    .draw(
                        &vertex_buffer,
                        &index_buffer,
                        &program,
                        &uniform! {
                            matrix: matrix,
                            time: time,
                        },
                        &params,
                    )
                    .unwrap();

                target.finish().unwrap();

                time += 0.016;
            }
            _ => (),
        }

        *control_flow = glutin::event_loop::ControlFlow::Poll;
    });
}
fn view_matrix(position: [f32; 3], direction: [f32; 3], up: [f32; 3]) -> [[f32; 4]; 4] {
    let f = {
        let f = [
            direction[0] - position[0],
            direction[1] - position[1],
            direction[2] - position[2],
        ];
        let len = f[0] * f[0] + f[1] * f[1] + f[2] * f[2];
        let len = len.sqrt();
        [f[0] / len, f[1] / len, f[2] / len]
    };

    let s = [
        up[1] * f[2] - up[2] * f[1],
        up[2] * f[0] - up[0] * f[2],
        up[0] * f[1] - up[1] * f[0],
    ];

    let s_norm = {
        let len = s[0] * s[0] + s[1] * s[1] + s[2] * s[2];
        let len = len.sqrt();
        [s[0] / len, s[1] / len, s[2] / len]
    };

    let u = [
        f[1] * s_norm[2] - f[2] * s_norm[1],
        f[2] * s_norm[0] - f[0] * s_norm[2],
        f[0] * s_norm[1] - f[1] * s_norm[0],
    ];

    let p = [
        -position[0] * s_norm[0] - position[1] * s_norm[1] - position[2] * s_norm[2],
        -position[0] * u[0] - position[1] * u[1] - position[2] * u[2],
        -position[0] * f[0] - position[1] * f[1] - position[2] * f[2],
    ];

    [
        [s_norm[0], u[0], f[0], 0.0],
        [s_norm[1], u[1], f[1], 0.0],
        [s_norm[2], u[2], f[2], 0.0],
        [p[0], p[1], p[2], 1.0],
    ]
}

fn multiply_matrix(a: [[f32; 4]; 4], b: [[f32; 4]; 4]) -> [[f32; 4]; 4] {
    let mut result = [[0.0; 4]; 4];
    for i in 0..4 {
        for j in 0..4 {
            result[i][j] =
                a[i][0] * b[0][j] + a[i][1] * b[1][j] + a[i][2] * b[2][j] + a[i][3] * b[3][j];
        }
    }
    result
}
