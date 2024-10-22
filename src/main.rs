extern crate glium;

use glium::glutin::event::{Event, WindowEvent};
use glium::glutin::event_loop::{ControlFlow, EventLoop};
use glium::glutin::window::WindowBuilder;
use glium::{glutin, implement_vertex, index::PrimitiveType, uniform, Surface};
use nalgebra::{Matrix4, Perspective3, Point3, Vector3};
use sha2::{Digest, Sha256};
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    input: String,
}

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
    color: [f32; 3],
}

implement_vertex!(Vertex, position, normal, color);

fn main() {
    let args = Cli::from_args();
    let hash = hash_input(&args.input);
    let (shape_vertices, indices, transformation) = derive_shape_and_transformation(&hash);

    let event_loop = EventLoop::new();
    let window_builder = WindowBuilder::new()
        .with_title("3D Shape Renderer")
        .with_inner_size(glutin::dpi::LogicalSize::new(800.0, 600.0));

    let context_builder = glutin::ContextBuilder::new()
        .with_depth_buffer(24)
        .with_vsync(true);

    let display = glium::Display::new(window_builder, context_builder, &event_loop)
        .expect("Failed to create display");

    let vertex_buffer = glium::VertexBuffer::new(&display, &shape_vertices).unwrap();
    let index_buffer =
        glium::IndexBuffer::new(&display, PrimitiveType::TrianglesList, &indices).unwrap();

    let program = glium::Program::from_source(
        &display,
        include_str!("vertex_shader.glsl"),
        include_str!("fragment_shader.glsl"),
        None,
    )
    .unwrap();

    // Camera setup
    let eye = Point3::new(2.0, 2.0, 2.0);
    let target = Point3::new(0.0, 0.0, 0.0);
    let up = Vector3::new(0.0, 1.0, 0.0);
    let view = Matrix4::look_at_rh(&eye, &target, &up);

    // Perspective projection
    let aspect_ratio = 800.0 / 600.0;
    let perspective = Perspective3::new(aspect_ratio, std::f32::consts::FRAC_PI_4, 0.1, 100.0);
    let perspective = perspective.to_homogeneous();

    let mut angle = 0.0f32;

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
                // Rotate the model
                angle += 0.01;
                let rotation = Matrix4::new_rotation(Vector3::new(0.0, angle, angle / 2.0));
                let model = transformation * rotation;

                //let _mvp = perspective * view * model;

                let light_position: [f32; 3] = [2.0, 2.0, 2.0];

                let mut target = display.draw();
                target.clear_color_and_depth((0.1, 0.1, 0.1, 1.0), 1.0);

                let uniforms = uniform! {
                    model_matrix: Into::<[[f32; 4]; 4]>::into(model),
                    view_matrix: Into::<[[f32; 4]; 4]>::into(view),
                    perspective_matrix: Into::<[[f32; 4]; 4]>::into(perspective),
                    light_position: light_position,
                };

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
                    .draw(&vertex_buffer, &index_buffer, &program, &uniforms, &params)
                    .unwrap();

                target.finish().unwrap();
            }
            _ => (),
        }
    });
}

fn hash_input(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input);
    format!("{:x}", hasher.finalize())
}

fn derive_shape_and_transformation(hash: &str) -> (Vec<Vertex>, Vec<u16>, Matrix4<f32>) {
    // SHA-256 hash produces 64 hexadecimal characters (32 bytes total)
    // Current hash usage (12 hex chars = 6 bytes):
    //
    // Bytes 0-1 (hex chars 0-1):   shape_type     "00"-"FF" -> 0-255 then mod 5
    // Bytes 2-3 (hex chars 2-3):   scale          "00"-"FF" -> 1.0-2.0 range
    // Bytes 4-5 (hex chars 4-5):   rotation       "00"-"FF" -> 0-360 degrees
    // Bytes 6-7 (hex chars 6-7):   red color      "00"-"FF" -> 0.0-1.0 range
    // Bytes 8-9 (hex chars 8-9):   green color    "00"-"FF" -> 0.0-1.0 range
    // Bytes 10-11 (hex chars 10-11): blue color   "00"-"FF" -> 0.0-1.0 range
    //
    // Unused: 52 hex chars (26 bytes) remaining for potential features:
    // - Surface texturing
    // - Vertex displacement
    // - Additional transformations
    // - Pattern generation
    // - Animation parameters
    // - Secondary shapes

    let shape_type = u8::from_str_radix(&hash[0..2], 16).unwrap() % 12;

    let mut scale = u8::from_str_radix(&hash[2..4], 16).unwrap() as f32 / 255.0;

    // Clamp scale to a reasonable range
    if scale < 0.1 {
        scale = 0.1;
    }

    if scale > 1.0 {
        scale = 1.0;
    }

    let rotation_angle = (u8::from_str_radix(&hash[4..6], 16).unwrap() as f32) * 360.0 / 255.0;

    // Generate base color from hash
    let r = u8::from_str_radix(&hash[6..8], 16).unwrap() as f32 / 255.0;
    let g = u8::from_str_radix(&hash[8..10], 16).unwrap() as f32 / 255.0;
    let b = u8::from_str_radix(&hash[10..12], 16).unwrap() as f32 / 255.0;
    let base_color = [r, g, b];

    let (vertices, indices) = match shape_type {
        0 => generate_cube(base_color),
        1 => generate_pyramid(base_color),
        2 => generate_octahedron(base_color),
        3 => generate_dodecahedron(base_color),
        4 => generate_icosahedron(base_color),
        5 => generate_prism(base_color, 6),
        6 => generate_star(base_color),
        7 => generate_torus(base_color, 20, 10),
        8 => generate_spiral_prism(base_color),
        9 => generate_gem(base_color),
        10 => generate_crystal(base_color),
        _ => generate_cube(base_color), // fallback to cube
    };

    let scale_matrix = Matrix4::new_scaling(scale);
    let rotation_matrix = Matrix4::new_rotation(Vector3::new(
        rotation_angle.to_radians(),
        (rotation_angle * 0.7).to_radians(),
        (rotation_angle * 0.3).to_radians(),
    ));

    (vertices, indices, scale_matrix * rotation_matrix)
}

fn calculate_normal(v1: [f32; 3], v2: [f32; 3], v3: [f32; 3]) -> [f32; 3] {
    let edge1 = [v2[0] - v1[0], v2[1] - v1[1], v2[2] - v1[2]];
    let edge2 = [v3[0] - v1[0], v3[1] - v1[1], v3[2] - v1[2]];
    let normal = [
        edge1[1] * edge2[2] - edge1[2] * edge2[1],
        edge1[2] * edge2[0] - edge1[0] * edge2[2],
        edge1[0] * edge2[1] - edge1[1] * edge2[0],
    ];
    let len = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
    [normal[0] / len, normal[1] / len, normal[2] / len]
}

fn color_variation(base_color: [f32; 3], variation: f32) -> [f32; 3] {
    [
        (base_color[0] + variation).min(1.0).max(0.0),
        (base_color[1] + variation).min(1.0).max(0.0),
        (base_color[2] + variation).min(1.0).max(0.0),
    ]
}

fn generate_cube(base_color: [f32; 3]) -> (Vec<Vertex>, Vec<u16>) {
    let positions = vec![
        [-1.0, -1.0, -1.0],
        [1.0, -1.0, -1.0],
        [1.0, 1.0, -1.0],
        [-1.0, 1.0, -1.0],
        [-1.0, -1.0, 1.0],
        [1.0, -1.0, 1.0],
        [1.0, 1.0, 1.0],
        [-1.0, 1.0, 1.0],
    ];

    let mut vertices = Vec::new();
    let indices = vec![
        0, 1, 2, 2, 3, 0, // front
        1, 5, 6, 6, 2, 1, // right
        5, 4, 7, 7, 6, 5, // back
        4, 0, 3, 3, 7, 4, // left
        3, 2, 6, 6, 7, 3, // top
        4, 5, 1, 1, 0, 4, // bottom
    ];

    // Create vertices with unique normals and varied colors for each face
    for i in 0..6 {
        let normal = match i {
            0 => [0.0, 0.0, -1.0], // front
            1 => [1.0, 0.0, 0.0],  // right
            2 => [0.0, 0.0, 1.0],  // back
            3 => [-1.0, 0.0, 0.0], // left
            4 => [0.0, 1.0, 0.0],  // top
            5 => [0.0, -1.0, 0.0], // bottom
            _ => unreachable!(),
        };

        let color = color_variation(base_color, (i as f32) * 0.1);

        for &idx in &indices[i * 6..(i + 1) * 6] {
            vertices.push(Vertex {
                position: positions[idx as usize],
                normal,
                color,
            });
        }
    }

    (vertices.clone(), (0..vertices.len() as u16).collect())
}

fn generate_pyramid(base_color: [f32; 3]) -> (Vec<Vertex>, Vec<u16>) {
    let positions = vec![
        [0.0, 1.0, 0.0],    // top
        [-1.0, -1.0, 1.0],  // front left
        [1.0, -1.0, 1.0],   // front right
        [1.0, -1.0, -1.0],  // back right
        [-1.0, -1.0, -1.0], // back left
    ];

    let mut vertices = Vec::new();
    let face_indices = vec![
        [0, 1, 2], // front
        [0, 2, 3], // right
        [0, 3, 4], // back
        [0, 4, 1], // left
        [1, 2, 3],
        [3, 4, 1], // base
    ];

    for (i, face) in face_indices.iter().enumerate() {
        let normal = calculate_normal(positions[face[0]], positions[face[1]], positions[face[2]]);
        let color = color_variation(base_color, (i as f32) * 0.15);

        for &idx in face {
            vertices.push(Vertex {
                position: positions[idx],
                normal,
                color,
            });
        }
    }

    (vertices.clone(), (0..vertices.len() as u16).collect())
}

fn generate_octahedron(base_color: [f32; 3]) -> (Vec<Vertex>, Vec<u16>) {
    let positions = vec![
        [0.0, 1.0, 0.0],  // top
        [1.0, 0.0, 0.0],  // right
        [0.0, 0.0, 1.0],  // front
        [-1.0, 0.0, 0.0], // left
        [0.0, 0.0, -1.0], // back
        [0.0, -1.0, 0.0], // bottom
    ];

    let mut vertices = Vec::new();
    let face_indices = vec![
        [0, 2, 1],
        [0, 3, 2],
        [0, 4, 3],
        [0, 1, 4], // top faces
        [5, 1, 2],
        [5, 2, 3],
        [5, 3, 4],
        [5, 4, 1], // bottom faces
    ];

    for (i, face) in face_indices.iter().enumerate() {
        let normal = calculate_normal(positions[face[0]], positions[face[1]], positions[face[2]]);
        let color = color_variation(base_color, (i as f32) * 0.12);

        for &idx in face {
            vertices.push(Vertex {
                position: positions[idx],
                normal,
                color,
            });
        }
    }

    (vertices.clone(), (0..vertices.len() as u16).collect())
}

fn generate_dodecahedron(base_color: [f32; 3]) -> (Vec<Vertex>, Vec<u16>) {
    let phi = (1.0 + 5.0f32.sqrt()) / 2.0;
    let positions = vec![
        [1.0, 1.0, 1.0],
        [1.0, 1.0, -1.0],
        [1.0, -1.0, 1.0],
        [1.0, -1.0, -1.0],
        [-1.0, 1.0, 1.0],
        [-1.0, 1.0, -1.0],
        [-1.0, -1.0, 1.0],
        [-1.0, -1.0, -1.0],
        [0.0, phi, 1.0 / phi],
        [0.0, phi, -1.0 / phi],
        [0.0, -phi, 1.0 / phi],
        [0.0, -phi, -1.0 / phi],
        [phi, 0.0, 1.0 / phi],
        [phi, 0.0, -1.0 / phi],
        [-phi, 0.0, 1.0 / phi],
        [-phi, 0.0, -1.0 / phi],
        [1.0 / phi, phi, 0.0],
        [1.0 / phi, -phi, 0.0],
        [-1.0 / phi, phi, 0.0],
        [-1.0 / phi, -phi, 0.0],
    ];

    // Normalize all vertices to create a more uniform shape
    let positions: Vec<[f32; 3]> = positions
        .iter()
        .map(|[x, y, z]| {
            let mag = (x * x + y * y + z * z).sqrt();
            [x / mag, y / mag, z / mag]
        })
        .collect();

    let face_indices = vec![
        [0, 8, 4, 14, 12],  // Face 1
        [0, 12, 2, 10, 8],  // Face 2
        [0, 16, 17, 2, 12], // Face 3
        [16, 1, 13, 3, 17], // Face 4
        [1, 9, 5, 15, 13],  // Face 5
        [9, 18, 19, 5, 15], // Face 6
        [18, 4, 14, 6, 19], // Face 7
        [14, 12, 2, 17, 6], // Face 8
        [17, 3, 11, 7, 6],  // Face 9
        [3, 13, 15, 5, 11], // Face 10
        [5, 19, 6, 7, 11],  // Face 11
        [4, 8, 10, 2, 14],  // Face 12
    ];

    let mut vertices = Vec::new();

    // Create vertices for each face with proper normals and varied colors
    for (i, face) in face_indices.iter().enumerate() {
        // Calculate face center for normal computation
        let center = [
            face.iter().map(|&idx| positions[idx][0]).sum::<f32>() / 5.0,
            face.iter().map(|&idx| positions[idx][1]).sum::<f32>() / 5.0,
            face.iter().map(|&idx| positions[idx][2]).sum::<f32>() / 5.0,
        ];

        // Use center as normal direction (will automatically point outward due to vertex ordering)
        let normal = {
            let len =
                (center[0] * center[0] + center[1] * center[1] + center[2] * center[2]).sqrt();
            [center[0] / len, center[1] / len, center[2] / len]
        };

        let color = color_variation(base_color, (i as f32) * 0.08);

        // Create triangles for the pentagon face
        for j in 1..4 {
            vertices.push(Vertex {
                position: positions[face[0]],
                normal,
                color,
            });
            vertices.push(Vertex {
                position: positions[face[j]],
                normal,
                color,
            });
            vertices.push(Vertex {
                position: positions[face[j + 1]],
                normal,
                color,
            });
        }
    }

    (vertices.clone(), (0..vertices.len() as u16).collect())
}

fn generate_icosahedron(base_color: [f32; 3]) -> (Vec<Vertex>, Vec<u16>) {
    let phi = (1.0 + 5.0f32.sqrt()) / 2.0;

    // Generate the 12 vertices of the icosahedron
    let positions = vec![
        [0.0, 1.0, phi],
        [0.0, -1.0, phi],
        [0.0, 1.0, -phi],
        [0.0, -1.0, -phi],
        [phi, 0.0, 1.0],
        [-phi, 0.0, 1.0],
        [phi, 0.0, -1.0],
        [-phi, 0.0, -1.0],
        [1.0, phi, 0.0],
        [-1.0, phi, 0.0],
        [1.0, -phi, 0.0],
        [-1.0, -phi, 0.0],
    ];

    // Normalize vertices
    let positions: Vec<[f32; 3]> = positions
        .iter()
        .map(|[x, y, z]| {
            let mag = (x * x + y * y + z * z).sqrt();
            [x / mag, y / mag, z / mag]
        })
        .collect();

    // Define the 20 triangular faces
    let face_indices = vec![
        [0, 1, 4],
        [0, 4, 8],
        [0, 8, 9],
        [0, 9, 5],
        [0, 5, 1],
        [3, 2, 6],
        [3, 6, 10],
        [3, 10, 11],
        [3, 11, 7],
        [3, 7, 2],
        [1, 5, 11],
        [5, 9, 7],
        [9, 8, 2],
        [8, 4, 6],
        [4, 1, 10],
        [10, 6, 4],
        [11, 10, 1],
        [7, 11, 5],
        [2, 7, 9],
        [6, 2, 8],
    ];

    let mut vertices = Vec::new();

    // Create vertices for each face with proper normals and varied colors
    for (i, face) in face_indices.iter().enumerate() {
        let normal = calculate_normal(positions[face[0]], positions[face[1]], positions[face[2]]);

        let color = color_variation(base_color, (i as f32) * 0.05);

        // Add vertices for the triangle
        for &idx in face {
            vertices.push(Vertex {
                position: positions[idx],
                normal,
                color,
            });
        }
    }

    (vertices.clone(), (0..vertices.len() as u16).collect())
}

/// Generate a regular prism with specified number of sides
fn generate_prism(base_color: [f32; 3], sides: u32) -> (Vec<Vertex>, Vec<u16>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    // Generate vertices for top and bottom faces
    for i in 0..sides {
        let angle = 2.0 * std::f32::consts::PI * (i as f32) / (sides as f32);
        let x = angle.cos();
        let z = angle.sin();

        // Bottom vertices
        vertices.push(Vertex {
            position: [x, -1.0, z],
            normal: [x, 0.0, z],
            color: color_variation(base_color, (i as f32) * 0.1),
        });

        // Top vertices
        vertices.push(Vertex {
            position: [x, 1.0, z],
            normal: [x, 0.0, z],
            color: color_variation(base_color, (i as f32) * 0.1),
        });
    }

    // Generate indices for the side faces
    for i in 0..sides {
        let next = (i + 1) % sides;
        let i0 = (i * 2) as u16;
        let i1 = (i * 2 + 1) as u16;
        let i2 = (next * 2) as u16;
        let i3 = (next * 2 + 1) as u16;

        // First triangle
        indices.push(i0);
        indices.push(i2);
        indices.push(i1);

        // Second triangle
        indices.push(i1);
        indices.push(i2);
        indices.push(i3);
    }

    // Generate indices for top and bottom faces
    let center_bottom = vertices.len() as u16;
    let center_top = center_bottom + 1;

    // Add center vertices
    vertices.push(Vertex {
        position: [0.0, -1.0, 0.0],
        normal: [0.0, -1.0, 0.0],
        color: base_color,
    });
    vertices.push(Vertex {
        position: [0.0, 1.0, 0.0],
        normal: [0.0, 1.0, 0.0],
        color: base_color,
    });

    // Add indices for bottom and top faces
    for i in 0..sides {
        let next = (i + 1) % sides;

        // Bottom face
        indices.push(center_bottom);
        indices.push((i * 2) as u16);
        indices.push((next * 2) as u16);

        // Top face
        indices.push(center_top);
        indices.push((i * 2 + 1) as u16);
        indices.push((next * 2 + 1) as u16);
    }

    (vertices, indices)
}

/// Generate a star-shaped polyhedron
fn generate_star(base_color: [f32; 3]) -> (Vec<Vertex>, Vec<u16>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    // Generate points of the star
    let points = 5;
    let inner_radius = 0.5;
    let outer_radius = 1.0;

    // Top point
    vertices.push(Vertex {
        position: [0.0, 1.0, 0.0],
        normal: [0.0, 1.0, 0.0],
        color: base_color,
    });

    // Generate outer and inner points
    for i in 0..points {
        let angle = 2.0 * std::f32::consts::PI * (i as f32) / (points as f32);
        let next_angle = 2.0 * std::f32::consts::PI * ((i + 1) as f32) / (points as f32);
        let mid_angle = (angle + next_angle) / 2.0;

        // Outer point
        let x = outer_radius * angle.cos();
        let z = outer_radius * angle.sin();
        vertices.push(Vertex {
            position: [x, 0.0, z],
            normal: normalize([x, 0.2, z]),
            color: color_variation(base_color, (i as f32) * 0.1),
        });

        // Inner point
        let x = inner_radius * mid_angle.cos();
        let z = inner_radius * mid_angle.sin();
        vertices.push(Vertex {
            position: [x, 0.0, z],
            normal: normalize([x, 0.2, z]),
            color: color_variation(base_color, (i as f32) * 0.15),
        });
    }

    // Bottom point
    vertices.push(Vertex {
        position: [0.0, -1.0, 0.0],
        normal: [0.0, -1.0, 0.0],
        color: base_color,
    });

    // Generate indices
    for i in 0..points {
        let outer = 1 + i * 2;
        let inner = 2 + i * 2;
        let next_outer = 1 + ((i + 1) % points) * 2;

        // Top triangles
        indices.extend_from_slice(&[0, outer as u16, inner as u16]);
        indices.extend_from_slice(&[0, inner as u16, next_outer as u16]);

        // Bottom triangles
        let bottom = vertices.len() as u16 - 1;
        indices.extend_from_slice(&[bottom, inner as u16, outer as u16]);
        indices.extend_from_slice(&[bottom, next_outer as u16, inner as u16]);
    }

    (vertices, indices)
}

/// Generate a torus
fn generate_torus(
    base_color: [f32; 3],
    major_segments: u32,
    minor_segments: u32,
) -> (Vec<Vertex>, Vec<u16>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    let major_radius = 0.7;
    let minor_radius = 0.3;

    // Generate vertices
    for i in 0..major_segments {
        let major_angle = 2.0 * std::f32::consts::PI * (i as f32) / (major_segments as f32);
        let major_x = major_angle.cos();
        let major_z = major_angle.sin();

        for j in 0..minor_segments {
            let minor_angle = 2.0 * std::f32::consts::PI * (j as f32) / (minor_segments as f32);
            let minor_x = minor_angle.cos();
            let minor_y = minor_angle.sin();

            let x = (major_radius + minor_radius * minor_x) * major_x;
            let y = minor_radius * minor_y;
            let z = (major_radius + minor_radius * minor_x) * major_z;

            let normal = normalize([minor_x * major_x, minor_y, minor_x * major_z]);

            vertices.push(Vertex {
                position: [x, y, z],
                normal,
                color: color_variation(base_color, (i as f32 * j as f32) * 0.02),
            });
        }
    }

    // Generate indices
    for i in 0..major_segments {
        for j in 0..minor_segments {
            let next_i = (i + 1) % major_segments;
            let next_j = (j + 1) % minor_segments;
            let current = (i * minor_segments + j) as u16;
            let next_major = (next_i * minor_segments + j) as u16;
            let next_minor = (i * minor_segments + next_j) as u16;
            let next_both = (next_i * minor_segments + next_j) as u16;

            indices.extend_from_slice(&[
                current, next_major, next_both, current, next_both, next_minor,
            ]);
        }
    }

    (vertices, indices)
}

/// Generate a spiral prism
fn generate_spiral_prism(base_color: [f32; 3]) -> (Vec<Vertex>, Vec<u16>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    let sides = 8;
    let spiral_segments = 16;
    let height_step = 2.0 / spiral_segments as f32;
    let twist_rate = 2.0 * std::f32::consts::PI;

    // Generate vertices
    for i in 0..=spiral_segments {
        let height = -1.0 + i as f32 * height_step;
        let twist = twist_rate * (i as f32 / spiral_segments as f32);

        for j in 0..sides {
            let angle = 2.0 * std::f32::consts::PI * (j as f32) / (sides as f32) + twist;
            let x = angle.cos() * (1.0 - (height * 0.2).abs());
            let z = angle.sin() * (1.0 - (height * 0.2).abs());

            let normal = normalize([x, height_step, z]);

            vertices.push(Vertex {
                position: [x, height, z],
                normal,
                color: color_variation(base_color, (i as f32 * j as f32) * 0.03),
            });
        }
    }

    // Generate indices
    for i in 0..spiral_segments {
        for j in 0..sides {
            let next_j = (j + 1) % sides;
            let current = (i * sides + j) as u16;
            let next_side = (i * sides + next_j) as u16;
            let next_segment = ((i + 1) * sides + j) as u16;
            let next_both = ((i + 1) * sides + next_j) as u16;

            indices.extend_from_slice(&[
                current,
                next_side,
                next_both,
                current,
                next_both,
                next_segment,
            ]);
        }
    }

    (vertices, indices)
}

/// Helper function to normalize a vector
fn normalize(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    [v[0] / len, v[1] / len, v[2] / len]
}

/// Generate a gem-like shape (continuation)
fn generate_gem(base_color: [f32; 3]) -> (Vec<Vertex>, Vec<u16>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    let sides = 8;
    let top_height = 1.0;
    let middle_height = 0.3;
    let bottom_height = -0.7;

    // Top point
    vertices.push(Vertex {
        position: [0.0, top_height, 0.0],
        normal: [0.0, 1.0, 0.0],
        color: color_variation(base_color, 0.2),
    });

    // Middle vertices
    for i in 0..sides {
        let angle = 2.0 * std::f32::consts::PI * (i as f32) / (sides as f32);
        let x = angle.cos();
        let z = angle.sin();

        // Upper middle vertices
        vertices.push(Vertex {
            position: [x * 0.7, middle_height, z * 0.7],
            normal: normalize([x, 0.5, z]),
            color: color_variation(base_color, (i as f32) * 0.1),
        });

        // Lower middle vertices
        vertices.push(Vertex {
            position: [x, 0.0, z],
            normal: normalize([x, 0.0, z]),
            color: color_variation(base_color, (i as f32) * 0.15),
        });

        // Bottom ring vertices
        vertices.push(Vertex {
            position: [x * 0.6, bottom_height, z * 0.6],
            normal: normalize([x, -0.5, z]),
            color: color_variation(base_color, (i as f32) * 0.05),
        });
    }

    // Bottom point
    vertices.push(Vertex {
        position: [0.0, bottom_height - 0.3, 0.0],
        normal: [0.0, -1.0, 0.0],
        color: base_color,
    });

    // Generate indices
    let bottom_center_index = vertices.len() as u16 - 1;

    // Top pyramids
    for i in 0..sides {
        let current_upper = 1 + i * 3;
        let next_upper = 1 + ((i + 1) % sides) * 3;

        // Top triangle
        indices.extend_from_slice(&[0, current_upper, next_upper]);

        // Upper middle section
        indices.extend_from_slice(&[current_upper, current_upper + 1, next_upper]);
        indices.extend_from_slice(&[next_upper, current_upper + 1, next_upper + 1]);

        // Lower middle section
        indices.extend_from_slice(&[current_upper + 1, current_upper + 2, next_upper + 1]);
        indices.extend_from_slice(&[next_upper + 1, current_upper + 2, next_upper + 2]);

        // Bottom pyramids
        indices.extend_from_slice(&[current_upper + 2, bottom_center_index, next_upper + 2]);
    }

    (vertices, indices)
}

/// Generate a crystalline shape
fn generate_crystal(base_color: [f32; 3]) -> (Vec<Vertex>, Vec<u16>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    let sides = 6;
    let height_segments = 4;
    let radius_variation = 0.3;

    // Generate vertices in rings with varying radii
    for h in 0..=height_segments {
        let height = 1.0 - (2.0 * h as f32 / height_segments as f32);
        let ring_radius = 1.0
            + radius_variation * (std::f32::consts::PI * h as f32 / height_segments as f32).sin();

        for i in 0..sides {
            let angle = 2.0 * std::f32::consts::PI * (i as f32) / (sides as f32);
            let x = ring_radius * angle.cos();
            let z = ring_radius * angle.sin();

            let normal = normalize([x, height * 0.5, z]);

            vertices.push(Vertex {
                position: [x, height, z],
                normal,
                color: color_variation(base_color, (h as f32 * 0.2) + (i as f32 * 0.1)),
            });
        }
    }

    // Generate indices
    for h in 0..height_segments {
        for i in 0..sides {
            let next_i = (i + 1) % sides;
            let current = (h * sides + i) as u16;
            let next_side = (h * sides + next_i) as u16;
            let next_height = ((h + 1) * sides + i) as u16;
            let next_both = ((h + 1) * sides + next_i) as u16;

            indices.extend_from_slice(&[
                current,
                next_side,
                next_both,
                current,
                next_both,
                next_height,
            ]);
        }
    }

    // Add top and bottom caps
    let top_center = vertices.len() as u16;
    let bottom_center = top_center + 1;

    vertices.push(Vertex {
        position: [0.0, 1.0, 0.0],
        normal: [0.0, 1.0, 0.0],
        color: color_variation(base_color, 0.3),
    });

    vertices.push(Vertex {
        position: [0.0, -1.0, 0.0],
        normal: [0.0, -1.0, 0.0],
        color: base_color,
    });

    // Top cap indices
    for i in 0..sides {
        let next = (i + 1) % sides;
        indices.extend_from_slice(&[top_center, i as u16, next as u16]);
    }

    // Bottom cap indices
    let bottom_ring_start = (height_segments * sides) as u16;
    for i in 0..sides {
        let next = (i + 1) % sides;
        indices.extend_from_slice(&[
            bottom_center,
            bottom_ring_start + next as u16,
            bottom_ring_start + i as u16,
        ]);
    }

    (vertices, indices)
}
