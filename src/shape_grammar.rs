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
    pub rotation: f32,
    pub base_color: [f32; 3],
    pub variations: [[f32; 16]; 3], // Changed to fixed-size arrays
}

#[derive(Debug)]
pub struct ShapeGrammar {
    pub base_shape: BaseShape,
    pub modifiers: Vec<Modifier>,
    pub parameters: ShapeParameters,
}

impl ShapeGrammar {
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
        let scale = 0.5 + (Self::safe_hash_value(hash, 1, 1, 128) as f32 / 255.0);
        let rotation = Self::safe_hash_value(hash, 2, 1, 128) as f32 * 360.0 / 255.0;

        let r = Self::safe_hash_value(hash, 3, 1, 128) as f32 / 255.0;
        let g = Self::safe_hash_value(hash, 4, 1, 128) as f32 / 255.0;
        let b = Self::safe_hash_value(hash, 5, 1, 128) as f32 / 255.0;

        let mut variations1 = [0.0; 16];
        let mut variations2 = [0.0; 16];
        let mut variations3 = [0.0; 16];

        for i in 0..16 {
            variations1[i] = Self::safe_hash_value(hash, 6 + i, 1, 128) as f32 / 255.0;
            variations2[i] = Self::safe_hash_value(hash, 22 + i, 1, 128) as f32 / 255.0;
            variations3[i] = Self::safe_hash_value(hash, 38 + i, 1, 128) as f32 / 255.0;
        }

        ShapeParameters {
            shape_type,
            scale,
            rotation,
            base_color: [r, g, b],
            variations: [variations1, variations2, variations3],
        }
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

        for i in 0..modifier_count {
            let modifier_type = hash[i + 1] % 4;
            let amount = (hash[i + 2] as f32 / 255.0) * 2.0;

            let modifier = match modifier_type {
                0 => Modifier::Twist { amount },
                1 => Modifier::Bend { amount },
                2 => Modifier::Round {
                    radius: amount * 0.2,
                },
                _ => Modifier::Pattern {
                    frequency: 5.0 + amount * 10.0,
                    amplitude: 0.1 + amount * 0.2,
                },
            };

            modifiers.push(modifier);
        }

        modifiers
    }
}
