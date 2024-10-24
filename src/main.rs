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
    // Get multiple hash values and combine them for better distribution
    let byte1 = safe_hash_value(hash, 0, 2, 128) as u32;
    let byte2 = safe_hash_value(hash, 20, 2, 128) as u32;
    let byte3 = safe_hash_value(hash, 40, 2, 128) as u32;

    // Combine the bytes by simply adding them and scaling
    let combined = byte1 + byte2 + byte3;

    // Scale the combined result to be within 0.0 to 40.0 range
    let shape_type = (combined as f32 / (255.0 * 3.0)) * 40.0;
    let shape_index = shape_type as usize;

    // Create dramatically different scale ranges based on shape type
    let scale_raw = safe_hash_value(hash, 2, 2, 128) as f32 / 255.0;
    let scale = match shape_index % 5 {
        0 => 0.3 + scale_raw * 0.7,                   // Smaller variations
        1 => 1.0 + scale_raw * 2.0,                   // Larger variations
        2 => 0.5 + scale_raw * scale_raw * 3.0,       // Exponential scaling
        3 => 1.0 / (0.5 + scale_raw),                 // Inverse scaling
        _ => 0.8 + (scale_raw - 0.5).powf(3.0) * 2.0, // Cubic scaling
    };

    // Varied rotation patterns
    let rot_base = safe_hash_value(hash, 4, 2, 128) as f32 / 255.0;
    let rotation = match shape_index % 4 {
        0 => rot_base * 720.0,                                // Double rotation
        1 => rot_base * rot_base * 360.0,                     // Quadratic rotation
        2 => (rot_base * std::f32::consts::PI).sin() * 360.0, // Sinusoidal
        _ => rot_base * 360.0,                                // Linear rotation
    };

    // More varied color generation
    let hue = safe_hash_value(hash, 6, 2, 128) as f32 / 255.0;
    let sat = 0.5 + (safe_hash_value(hash, 8, 2, 128) as f32 / 255.0) * 0.5;
    let val = 0.4 + (safe_hash_value(hash, 10, 2, 128) as f32 / 255.0) * 0.6;
    let [r, g, b] = hsv_to_rgb(hue, sat, val);

    let mut variations1 = [0.0; 16];
    let mut variations2 = [0.0; 16];
    let mut variations3 = [0.0; 8];

    // Shape-specific variation patterns
    let shape_category = shape_index / 5; // Group shapes into categories

    // Fill variations1 with shape-dependent distributions
    for i in 0..16 {
        let base = safe_hash_value(hash, 12 + i * 2, 2, 128) as f32 / 255.0;

        variations1[i] = match shape_category {
            0 => {
                // Geometric primitives (spheres, boxes, etc.)
                match i {
                    0..=2 => 0.5 + (base - 0.5).powf(3.0) * 2.0, // Dramatic size changes
                    3..=4 => base * base * 3.0,                  // Strong deformation
                    _ => base.powf(0.3),                         // Subtle other parameters
                }
            }
            1 => {
                // Complex shapes (fractals, etc.)
                match i {
                    0..=2 => 0.3 + base * 1.7,                  // Wide size range
                    3..=6 => ((base * 6.28).sin() + 1.0) * 0.8, // Oscillating deformations
                    _ => base.sqrt() * 2.0,                     // Moderate other params
                }
            }
            2 => {
                // Organic shapes
                match i {
                    0..=2 => 1.0 - (1.0 - base * base) * 0.8, // Inverse square scaling
                    3..=8 => 0.2 + (base * std::f32::consts::PI).sin() * 0.8, // Sinusoidal variation
                    _ => base.powf(1.5), // Enhanced other parameters
                }
            }
            3 => {
                // Mechanical shapes
                match i {
                    0..=2 => (base * 4.0).floor() / 3.0, // Quantized sizes
                    3..=6 => base.powf(0.3) * 1.5,       // Strong deformations
                    _ => 0.2 + base * 0.8,               // Linear other params
                }
            }
            _ => {
                // Abstract shapes
                match i {
                    0..=4 => ((base * 8.0).sin() + 1.0) / 2.0 * 2.0, // Highly oscillating
                    5..=8 => base.powf(2.0) * 2.0,                   // Quadratic scaling
                    _ => 0.3 + (base * base * base) * 0.7,           // Cubic other params
                }
            }
        };
    }

    // Fill variations2 with contrasting patterns
    for i in 0..16 {
        let base = safe_hash_value(hash, 44 + i * 2, 2, 128) as f32 / 255.0;
        let phase = base * std::f32::consts::PI * 2.0;

        variations2[i] = match shape_category {
            0 => 1.0 - variations1[i],             // Inverse of variations1
            1 => (phase.sin() + 1.0) / 2.0 * 1.5,  // Sinusoidal
            2 => base.powf(0.5) * 2.0,             // Square root
            3 => (base * 4.0).floor() / 3.0,       // Quantized
            _ => ((base * 6.0).sin() + 1.0) / 2.0, // High-frequency oscillation
        };
    }

    // Fill variations3 with fine detail modulations
    for i in 0..8 {
        let base = safe_hash_value(hash, 76 + i * 2, 2, 128) as f32 / 255.0;
        let multiplier = 1.0 + (shape_index as f32 * 0.1); // Shape-dependent scaling

        variations3[i] = match i {
            0..=2 => base.powf(0.3) * multiplier, // Stronger effect for higher indices
            3..=5 => ((base * 8.0).sin() + 1.0) / 2.0 * multiplier,
            _ => (0.2 + base * 0.8) * multiplier,
        };
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

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> [f32; 3] {
    let c = v * s;
    let h = h * 6.0;
    let x = c * (1.0 - ((h % 2.0) - 1.0).abs());
    let m = v - c;

    let (r, g, b) = match h as i32 {
        0 | 6 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        5 => (c, 0.0, x),
        _ => (0.0, 0.0, 0.0),
    };

    [r + m, g + m, b + m]
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
    let _fragment_shader = include_str!("fragment_shader_init.glsl").to_string()
        + "\n"
        + include_str!("fragment_shader_helpers.glsl")
        + "\n"
        + include_str!("fragment_shader_shapes.glsl")
        + "\n"
        + include_str!("fragment_shader_main.glsl");
    let debug = include_str!("debug.glsl");

    let program =
        glium::Program::from_source(&display, include_str!("vertex_shader.glsl"), &debug, None)
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

                // log the shape type
                println!("Shape type: {}", shape_type);

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
