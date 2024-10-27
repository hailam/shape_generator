#[derive(Debug, Clone)]
pub enum BaseShape {
    Sphere,
    Box,
    Torus,
    Cylinder,
    Cone,
    Capsule,
    Prism,
}

#[derive(Debug, Clone)]
pub enum Modifier {
    Twist { amount: f32 },
    Bend { amount: f32 },
    Round { radius: f32 },
    Pattern { frequency: f32, amplitude: f32 },
}

#[derive(Debug, Clone)]
pub struct ShapeParameters {
    pub shape_type: f32,
    pub scale: f32,
    pub rotation: [f32; 4],
    pub base_color: [f32; 3],
    pub variations: [[f32; 16]; 3],
    pub golden_ratio_factors: [f32; 4], // Added for golden ratio proportions
}

#[derive(Debug)]
pub struct ShapeGrammar {
    pub base_shape: BaseShape,
    pub modifiers: Vec<Modifier>,
    pub parameters: ShapeParameters,
}

impl ShapeGrammar {
    const PHI: f32 = 1.61803398875;
    const PHI_INVERSE: f32 = 0.61803398875;
    const TARGET_SCALE: f32 = 0.6;
    const MIN_SCALE: f32 = 0.4;
    const MAX_SCALE: f32 = 0.8;

    pub fn from_hash(input: &str) -> Self {
        let hash = blake3::hash(input.as_bytes());
        let hash_bytes = hash.as_bytes();

        let params = Self::generate_parameters(hash_bytes);
        let base_shape = Self::determine_base_shape(&params);
        let modifiers = Self::generate_modifiers(hash_bytes);

        Self {
            base_shape,
            modifiers,
            parameters: params,
        }
    }

    fn safe_hash_value(hash: &[u8], start: usize, length: usize, default: u8) -> u8 {
        if start + length <= hash.len() {
            hash[start]
        } else {
            default
        }
    }

    fn generate_parameters(hash: &[u8]) -> ShapeParameters {
        let shape_type = (Self::safe_hash_value(hash, 0, 1, 128) as f32 / 255.0) * 3.0;

        // Normalize base scale to target range
        let raw_scale = Self::safe_hash_value(hash, 1, 1, 128) as f32 / 255.0;
        let normalized_scale = Self::MIN_SCALE + (Self::MAX_SCALE - Self::MIN_SCALE) * raw_scale;
        let scale_factor = Self::TARGET_SCALE / normalized_scale;
        let scale = (normalized_scale * scale_factor) * Self::PHI_INVERSE;

        // Create rotation angles based on golden ratio subdivisions
        let base_rotation = Self::safe_hash_value(hash, 2, 1, 128) as f32 * 360.0 / 255.0;
        let rotation = [
            base_rotation,
            base_rotation * Self::PHI_INVERSE,
            base_rotation * Self::PHI_INVERSE * Self::PHI_INVERSE,
            base_rotation * Self::PHI_INVERSE * Self::PHI_INVERSE * Self::PHI_INVERSE,
        ];

        // Generate harmonious colors using golden ratio
        let hue = Self::safe_hash_value(hash, 3, 1, 128) as f32 / 255.0;
        let base_color = Self::generate_harmonious_color(hue);

        // Generate variations with controlled frequency
        let mut variations = [[0.0; 16]; 3];
        for buffer in 0..3 {
            for i in 0..16 {
                let raw_value =
                    Self::safe_hash_value(hash, 6 + buffer * 16 + i, 1, 128) as f32 / 255.0;
                variations[buffer][i] =
                    Self::smooth_step(raw_value) * (1.0 / Self::PHI.powi(buffer as i32 + 1));
            }
        }

        // Generate golden ratio based factors for modifiers
        let golden_ratio_factors = [
            Self::PHI_INVERSE,
            Self::PHI_INVERSE * Self::PHI_INVERSE,
            Self::PHI_INVERSE * Self::PHI_INVERSE * Self::PHI_INVERSE,
            Self::PHI_INVERSE * Self::PHI_INVERSE * Self::PHI_INVERSE * Self::PHI_INVERSE,
        ];

        ShapeParameters {
            shape_type,
            scale,
            rotation,
            base_color,
            variations,
            golden_ratio_factors,
        }
    }

    fn smooth_step(x: f32) -> f32 {
        // Smoothstep function to reduce sharp transitions
        let t = (x - 0.0) / (1.0 - 0.0);
        if t <= 0.0 {
            return 0.0;
        }
        if t >= 1.0 {
            return 1.0;
        }
        t * t * (3.0 - 2.0 * t)
    }

    fn generate_harmonious_color(hue: f32) -> [f32; 3] {
        // Convert hue to RGB using golden ratio based offsets
        let h1 = hue;
        let h2 = (h1 + Self::PHI_INVERSE) % 1.0;
        let h3 = (h1 + 2.0 * Self::PHI_INVERSE) % 1.0;

        [
            0.5 + 0.5 * f32::cos(h1 * std::f32::consts::PI * 2.0),
            0.5 + 0.5 * f32::cos(h2 * std::f32::consts::PI * 2.0),
            0.5 + 0.5 * f32::cos(h3 * std::f32::consts::PI * 2.0),
        ]
    }

    fn determine_base_shape(params: &ShapeParameters) -> BaseShape {
        match params.shape_type as usize {
            0 => BaseShape::Sphere,
            1 => BaseShape::Box,
            2 => BaseShape::Torus,
            3 => BaseShape::Cylinder,
            4 => BaseShape::Cone,
            5 => BaseShape::Capsule,
            6 => BaseShape::Prism,
            _ => BaseShape::Torus,
        }
    }

    fn generate_modifiers(hash: &[u8]) -> Vec<Modifier> {
        let mut modifiers = Vec::new();
        let modifier_count = (hash[0] % 3) as usize + 1;

        // track impact
        let total_scale_impact = 1.0;

        for i in 0..modifier_count {
            let modifier_type = hash[i + 1] % 4;
            // Use golden ratio for modifier parameters
            let mut amount = (hash[i + 2] as f32 / 255.0) * Self::PHI_INVERSE.powi(i as i32 + 1);

            // Normalize modifier amounts to maintain scale
            amount *= Self::TARGET_SCALE / total_scale_impact;

            let modifier = match modifier_type {
                0 => Modifier::Twist {
                    amount: amount * std::f32::consts::PI,
                },
                1 => Modifier::Bend {
                    amount: amount * Self::PHI,
                },
                2 => Modifier::Round {
                    radius: amount * 0.2 * Self::PHI_INVERSE,
                },
                _ => Modifier::Pattern {
                    frequency: 5.0 + amount * 10.0 * Self::PHI,
                    amplitude: 0.1 + amount * 0.2 * Self::PHI_INVERSE,
                },
            };

            modifiers.push(modifier);
        }

        modifiers
    }
}
