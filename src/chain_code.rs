use glium::implement_vertex;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::f32::consts::PI;

const FRAC_1_SQRT_2: f32 = 0.70710678118654752440084436210485;
const FRAC_1_SQRT_3: f32 = 0.57735026918962576450914878050196;

#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub color: [f32; 3],
    pub texture_coord: [f32; 2],
    pub pen_type: u32,
}
implement_vertex!(Vertex, position, normal, color, texture_coord, pen_type);

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction3D {
    // Face neighbors (6)
    Right,
    Left,
    Up,
    Down,
    Front,
    Back,

    // Edge neighbors (12)
    UpRight,
    UpLeft,
    UpFront,
    UpBack,
    DownRight,
    DownLeft,
    DownFront,
    DownBack,
    FrontRight,
    FrontLeft,
    BackRight,
    BackLeft,

    // Vertex neighbors (8)
    UpFrontRight,
    UpFrontLeft,
    UpBackRight,
    UpBackLeft,
    DownFrontRight,
    DownFrontLeft,
    DownBackRight,
    DownBackLeft,
}

pub struct ChainCodeGenerator {
    position: [f32; 3],
    grid_size: f32,
    bounds: ([f32; 3], [f32; 3]),
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    previous_directions: Vec<Direction3D>,
    center_tendency: f32,
    rng: StdRng,
}

impl ChainCodeGenerator {
    pub fn new(grid_size: f32, bounds: ([f32; 3], [f32; 3])) -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            grid_size,
            bounds,
            vertices: Vec::new(),
            indices: Vec::new(),
            previous_directions: Vec::new(),
            center_tendency: 0.3,
            rng: StdRng::seed_from_u64(42),
        }
    }

    pub fn generate(&mut self, input: &[u8]) -> (Vec<Vertex>, Vec<u32>) {
        self.vertices.clear();
        self.indices.clear();
        self.previous_directions.clear();
        self.position = [0.0, 0.0, 0.0];

        let directions = self.bytes_to_directions(input);
        let mut prev_position = self.position;

        for (i, dir) in directions.iter().enumerate() {
            let movement = self.direction_to_vector(*dir);
            let next_position = [
                prev_position[0] + movement[0],
                prev_position[1] + movement[1],
                prev_position[2] + movement[2],
            ];

            if self.is_within_bounds(next_position) {
                let progress = i as f32 / directions.len() as f32;
                self.add_tube_segment(prev_position, next_position, progress);
                prev_position = next_position;
                self.position = next_position;
                self.previous_directions.push(*dir);
            }
        }

        // Normalize vertices before returning
        self.normalize_vertices();

        (self.vertices.clone(), self.indices.clone())
    }

    fn bytes_to_directions(&mut self, bytes: &[u8]) -> Vec<Direction3D> {
        let mut directions = Vec::new();

        for window in bytes.windows(2) {
            let _combined = ((window[0] as u16) << 8) | window[1] as u16;
            let available_dirs = self.get_available_directions();

            if let Some(dirs) = available_dirs {
                if dirs.is_empty() {
                    continue;
                }

                let selected_dir = self.select_best_direction(&dirs);
                directions.push(selected_dir);
            }
        }

        directions
    }

    fn get_available_directions(&self) -> Option<Vec<Direction3D>> {
        use Direction3D::*;
        let all_directions = vec![
            Right,
            Left,
            Up,
            Down,
            Front,
            Back,
            UpRight,
            UpLeft,
            UpFront,
            UpBack,
            DownRight,
            DownLeft,
            DownFront,
            DownBack,
            FrontRight,
            FrontLeft,
            BackRight,
            BackLeft,
            UpFrontRight,
            UpFrontLeft,
            UpBackRight,
            UpBackLeft,
            DownFrontRight,
            DownFrontLeft,
            DownBackRight,
            DownBackLeft,
        ];

        let valid_directions: Vec<Direction3D> = all_directions
            .into_iter()
            .filter(|&dir| {
                let next_pos = self.calculate_next_position(dir);
                self.is_within_bounds(next_pos)
            })
            .collect();

        if valid_directions.is_empty() {
            None
        } else {
            Some(valid_directions)
        }
    }

    fn calculate_direction_score(&mut self, dir: Direction3D, next_pos: [f32; 3]) -> f32 {
        let mut score = 0.0;

        // Prefer directions that keep the shape within bounds
        let bounds_score = if self.is_within_bounds(next_pos) {
            1.0
        } else {
            0.0
        };
        score += bounds_score * 2.0;

        // Tendency to return to center when near bounds
        let center_dist = (next_pos[0].powi(2) + next_pos[1].powi(2) + next_pos[2].powi(2)).sqrt();
        let center_score = 1.0 / (1.0 + center_dist);
        score += center_score * self.center_tendency;

        // Encourage folding by preferring directions similar to recent ones
        if let Some(prev_dir) = self.previous_directions.last() {
            let continuity_score = if *prev_dir == dir { 0.5 } else { 0.0 };
            score += continuity_score;
        }

        // Add some randomness to prevent getting stuck in patterns
        score += self.rng.gen::<f32>() * 0.1;

        score
    }

    fn select_best_direction(&mut self, available_dirs: &[Direction3D]) -> Direction3D {
        available_dirs
            .iter()
            .map(|&dir| {
                let next_pos = self.calculate_next_position(dir);
                let score = self.calculate_direction_score(dir, next_pos);
                (dir, score)
            })
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(dir, _)| dir)
            .unwrap_or(Direction3D::Up)
    }

    fn calculate_next_position(&self, dir: Direction3D) -> [f32; 3] {
        let movement = self.direction_to_vector(dir);
        [
            self.position[0] + movement[0],
            self.position[1] + movement[1],
            self.position[2] + movement[2],
        ]
    }

    fn normalize_vertices(&mut self) {
        if self.vertices.is_empty() {
            return;
        }

        // Find current bounds
        let mut min_pos = [f32::MAX, f32::MAX, f32::MAX];
        let mut max_pos = [f32::MIN, f32::MIN, f32::MIN];

        for vertex in &self.vertices {
            for i in 0..3 {
                min_pos[i] = min_pos[i].min(vertex.position[i]);
                max_pos[i] = max_pos[i].max(vertex.position[i]);
            }
        }

        // Calculate center
        let center = [
            (min_pos[0] + max_pos[0]) * 0.5,
            (min_pos[1] + max_pos[1]) * 0.5,
            (min_pos[2] + max_pos[2]) * 0.5,
        ];

        // Calculate maximum extent from center
        let mut max_extent = 0.0f32;
        for vertex in &self.vertices {
            let dist = [
                (vertex.position[0] - center[0]).abs(),
                (vertex.position[1] - center[1]).abs(),
                (vertex.position[2] - center[2]).abs(),
            ];
            max_extent = max_extent.max(dist[0].max(dist[1].max(dist[2])));
        }

        // Scale to fit in [-0.8, 0.8] cube
        let scale = if max_extent > 0.0 {
            0.8 / max_extent
        } else {
            1.0
        };

        // Normalize vertices
        for vertex in &mut self.vertices {
            for i in 0..3 {
                vertex.position[i] = (vertex.position[i] - center[i]) * scale;
            }
            // Also normalize normal vectors (in case they're not unit length)
            let len = (vertex.normal[0] * vertex.normal[0]
                + vertex.normal[1] * vertex.normal[1]
                + vertex.normal[2] * vertex.normal[2])
                .sqrt();
            if len > 0.0 {
                for i in 0..3 {
                    vertex.normal[i] /= len;
                }
            }
        }

        // Verify normalization
        let mut new_min = [f32::MAX, f32::MAX, f32::MAX];
        let mut new_max = [f32::MIN, f32::MIN, f32::MIN];
        for vertex in &self.vertices {
            for i in 0..3 {
                new_min[i] = new_min[i].min(vertex.position[i]);
                new_max[i] = new_max[i].max(vertex.position[i]);
            }
        }
        println!(
            "Normalized bounds: x=[{:.3}, {:.3}], y=[{:.3}, {:.3}], z=[{:.3}, {:.3}]",
            new_min[0], new_max[0], new_min[1], new_max[1], new_min[2], new_max[2]
        );
    }

    fn direction_to_vector(&self, dir: Direction3D) -> [f32; 3] {
        let step = self.grid_size;
        let diag_step = step * FRAC_1_SQRT_2;
        let tri_step = step * FRAC_1_SQRT_3;

        match dir {
            Direction3D::Right => [step, 0.0, 0.0],
            Direction3D::Left => [-step, 0.0, 0.0],
            Direction3D::Up => [0.0, step, 0.0],
            Direction3D::Down => [0.0, -step, 0.0],
            Direction3D::Front => [0.0, 0.0, step],
            Direction3D::Back => [0.0, 0.0, -step],
            Direction3D::UpRight => [diag_step, diag_step, 0.0],
            Direction3D::UpLeft => [-diag_step, diag_step, 0.0],
            Direction3D::UpFront => [0.0, diag_step, diag_step],
            Direction3D::UpBack => [0.0, diag_step, -diag_step],
            Direction3D::DownRight => [diag_step, -diag_step, 0.0],
            Direction3D::DownLeft => [-diag_step, -diag_step, 0.0],
            Direction3D::DownFront => [0.0, -diag_step, diag_step],
            Direction3D::DownBack => [0.0, -diag_step, -diag_step],
            Direction3D::FrontRight => [diag_step, 0.0, diag_step],
            Direction3D::FrontLeft => [-diag_step, 0.0, diag_step],
            Direction3D::BackRight => [diag_step, 0.0, -diag_step],
            Direction3D::BackLeft => [-diag_step, 0.0, -diag_step],
            Direction3D::UpFrontRight => [tri_step, tri_step, tri_step],
            Direction3D::UpFrontLeft => [-tri_step, tri_step, tri_step],
            Direction3D::UpBackRight => [tri_step, tri_step, -tri_step],
            Direction3D::UpBackLeft => [-tri_step, tri_step, -tri_step],
            Direction3D::DownFrontRight => [tri_step, -tri_step, tri_step],
            Direction3D::DownFrontLeft => [-tri_step, -tri_step, tri_step],
            Direction3D::DownBackRight => [tri_step, -tri_step, -tri_step],
            Direction3D::DownBackLeft => [-tri_step, -tri_step, -tri_step],
        }
    }

    fn is_within_bounds(&self, pos: [f32; 3]) -> bool {
        pos[0] >= self.bounds.0[0]
            && pos[0] <= self.bounds.1[0]
            && pos[1] >= self.bounds.0[1]
            && pos[1] <= self.bounds.1[1]
            && pos[2] >= self.bounds.0[2]
            && pos[2] <= self.bounds.1[2]
    }

    fn add_tube_segment(&mut self, start: [f32; 3], end: [f32; 3], t: f32) {
        let segments = 12;
        let base_radius = self.grid_size * 0.15;

        let dir = normalize([end[0] - start[0], end[1] - start[1], end[2] - start[2]]);

        let (right, up) = create_coordinate_system(dir);

        // More varied color generation
        let base_color = hsv_to_rgb(
            t * 360.0,                   // Hue varies with progress
            0.7 + (t * 5.0).sin() * 0.3, // Saturation varies
            0.8 + (t * 7.0).cos() * 0.2, // Value varies
        );

        let base_idx = self.vertices.len() as u32;

        for i in 0..segments {
            let angle = (i as f32 / segments as f32) * 2.0 * PI;
            let next_angle = ((i + 1) as f32 / segments as f32) * 2.0 * PI;

            // Add more variation to radius
            let radius_var = base_radius * (1.0 + 0.15 * (angle * 3.0 + t * 10.0).sin());
            let next_radius_var = base_radius * (1.0 + 0.15 * (next_angle * 3.0 + t * 10.0).sin());

            for &(radius, pos, v) in &[(radius_var, start, 0.0), (next_radius_var, end, 1.0)] {
                let wave = 0.02 * (angle * 4.0 + t * 10.0).sin();
                let offset = [
                    right[0] * angle.cos() + up[0] * angle.sin(),
                    right[1] * angle.cos() + up[1] * angle.sin(),
                    right[2] * angle.cos() + up[2] * angle.sin(),
                ];

                // Vary color slightly based on position
                let color_variation = 0.1 * (pos[0].sin() + pos[1].cos() + pos[2].sin());
                let varied_color = [
                    (base_color[0] + color_variation).clamp(0.0, 1.0),
                    (base_color[1] + color_variation).clamp(0.0, 1.0),
                    (base_color[2] + color_variation).clamp(0.0, 1.0),
                ];

                self.vertices.push(Vertex {
                    position: [
                        pos[0] + offset[0] * radius * (1.0 + wave),
                        pos[1] + offset[1] * radius * (1.0 + wave),
                        pos[2] + offset[2] * radius * (1.0 + wave),
                    ],
                    normal: offset,
                    color: varied_color,
                    texture_coord: [angle / (2.0 * PI), v],
                    pen_type: ((t * 5.0) as u32) % 5,
                });
            }
        }

        // Generate indices
        for i in 0..segments {
            let i0 = base_idx + i * 2;
            let i1 = base_idx + (i * 2 + 1);
            let i2 = base_idx + ((i * 2 + 2) % (segments * 2));
            let i3 = base_idx + ((i * 2 + 3) % (segments * 2));

            self.indices.extend_from_slice(&[i0, i1, i2]);
            self.indices.extend_from_slice(&[i2, i1, i3]);
        }
    }
}

fn normalize(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len == 0.0 {
        v
    } else {
        [v[0] / len, v[1] / len, v[2] / len]
    }
}

fn create_coordinate_system(forward: [f32; 3]) -> ([f32; 3], [f32; 3]) {
    let right = if forward[1].abs() < 0.999 {
        normalize([forward[2], 0.0, -forward[0]])
    } else {
        [1.0, 0.0, 0.0]
    };

    let up = [
        forward[1] * right[2] - forward[2] * right[1],
        forward[2] * right[0] - forward[0] * right[2],
        forward[0] * right[1] - forward[1] * right[0],
    ];

    (normalize(right), normalize(up))
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> [f32; 3] {
    let h = h % 360.0;
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;

    let (r, g, b) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    [r + m, g + m, b + m]
}
