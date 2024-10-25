#version 140

in vec2 v_position;
out vec4 color;
uniform float time;
uniform float base_radius;
uniform vec3 base_color;
uniform float deform_factor;
uniform float speed;
uniform uint pattern;
uniform float metallic;
uniform float roughness;

const float PI = 3.14159265359;
const float PHI = 1.61803398875;
const float TAU = PI * 2.0;

float smin(float a, float b, float k) {
    float h = clamp(0.5 + 0.5 * (b - a) / k, 0.0, 1.0);
    return mix(b, a, h) - k * h * (1.0 - h);
}

mat2 rot2D(float angle) {
    float s = sin(angle);
    float c = cos(angle);
    return mat2(c, -s, s, c);
}

// 3D rotation matrices
mat3 rotateX(float angle) {
    float s = sin(angle);
    float c = cos(angle);
    return mat3(
        1.0, 0.0, 0.0,
        0.0, c, -s,
        0.0, s, c
    );
}

mat3 rotateY(float angle) {
    float s = sin(angle);
    float c = cos(angle);
    return mat3(
        c, 0.0, s,
        0.0, 1.0, 0.0,
        -s, 0.0, c
    );
}

mat3 rotateZ(float angle) {
    float s = sin(angle);
    float c = cos(angle);
    return mat3(
        c, -s, 0.0,
        s, c, 0.0,
        0.0, 0.0, 1.0
    );
}

// Improved 3D conversion functions
vec3 to3D(vec2 p, float d) {
    // Reduced height factor to prevent extreme depth changes
    float height = smoothstep(0.0, 0.1, -d) * 0.2;
    return vec3(p, height);
}

// Improved perspective projection
vec2 to2D(vec3 p) {
    // Adjust perspective division factor
    float perspective = 0.2; // Smaller value for less extreme perspective
    return p.xy / (1.0 + p.z * perspective);
}

float fibonacciSpiral(vec2 p, float scale) {
    float angle = atan(p.y, p.x);
    float radius = length(p);
    float spiral = log(radius) / PHI - angle;
    return abs(mod(spiral, TAU/6.0) - TAU/12.0) * scale;
}

float sdCircle(vec2 p, float r) {
    return length(p) - r;
}

float sdPentagon(vec2 p, float r) {
    const vec3 k = vec3(0.809016994, 0.587785252, 0.726542528);
    p.x = abs(p.x);
    p -= 2.0 * min(dot(vec2(-k.x, k.y), p), 0.0) * vec2(-k.x, k.y);
    p -= 2.0 * min(dot(vec2(k.x, k.y), p), 0.0) * vec2(k.x, k.y);
    p -= vec2(clamp(p.x, -r * k.z, r * k.z), r);
    return length(p) * sign(p.y);
}

// Secondary grammar patterns
float secondaryPattern(vec2 p, float d, float deformAmount) {
    // Use deform_factor to select and mix patterns
    float patternSelect = mod(deformAmount * 5.0, 4.0);
    float intensity = deformAmount * 0.3; // Control overall deformation strength
    
    // Base frequency for patterns
    float freq = 5.0 + deformAmount * 10.0;
    
    if (patternSelect < 1.0) {
        // Logarithmic spiral ridges
        float angle = atan(p.y, p.x);
        float radius = length(p);
        float spiral = sin(log(radius) * freq + angle * PHI) * intensity;
        d += spiral * smoothstep(0.0, 0.1, -d);
    }
    else if (patternSelect < 2.0) {
        // Fibonacci lattice
        vec2 fib = vec2(PHI, PHI * PHI);
        float lat = sin(dot(p, fib) * freq) * intensity;
        d += lat * smoothstep(0.0, 0.1, -d);
    }
    else if (patternSelect < 3.0) {
        // Golden rectangle grid
        vec2 grid = mod(p * freq, PHI) - PHI/2.0;
        float gridPattern = length(grid) * intensity;
        d -= gridPattern * smoothstep(0.0, 0.1, -d);
    }
    else {
        // Pentagonal tiling
        float angle = atan(p.y, p.x) * 5.0;
        float radius = length(p);
        float penta = sin(angle + radius * freq) * intensity;
        d += penta * smoothstep(0.0, 0.1, -d);
    }
    
    return d;
}

float sdf(vec2 p) {
    // Calculate initial shape
    float d = sdCircle(p, base_radius);
    
    // Convert to 3D with controlled depth
    vec3 p3 = to3D(p, d);
    
    // Slower, more subtle rotation
    float slowTime = time * 0.05;
    
    // Reduced rotation amplitudes
    mat3 rot = rotateY(slowTime) * 
               rotateX(sin(slowTime * 0.5) * 0.05) * 
               rotateZ(cos(slowTime * 0.3) * 0.05);
    
    // Apply rotation to 3D point
    p3 = rot * p3;
    
    // Project back to 2D with controlled perspective
    p = to2D(p3);
    
    // Primary shape grammar
    d = sdCircle(p, base_radius);
    
    if (pattern == 0u) {
        float spiral = fibonacciSpiral(p, 0.3);
        float spiralDetail = smoothstep(0.0, 0.1, spiral) * 0.1;
        d -= spiralDetail;
        
        float penta = sdPentagon(p, base_radius * 0.8);
        d = smin(d, penta, 0.1);
    }
    else if (pattern == 1u) {
        for (int i = 0; i < 5; i++) {
            float scale = pow(1.0/PHI, float(i));
            vec2 q = rot2D(float(i) * PI/5.0) * p;
            float rect = length(max(abs(q) - vec2(base_radius * scale, base_radius * scale/PHI), 0.0));
            d = smin(d, rect, 0.15);
        }
    }
    else if (pattern == 2u) {
        for (int i = 0; i < 5; i++) {
            float a = float(i) * TAU/5.0;
            vec2 q = rot2D(a) * p;
            float arm = length(q - vec2(base_radius/PHI, 0.0)) - base_radius * 0.2;
            d = smin(d, arm, 0.15);
        }
    }
    else {
        for (int i = 0; i < 3; i++) {
            float scale = pow(1.0/PHI, float(i));
            vec2 q = rot2D(float(i) * PI/3.0) * p;
            float penta = sdPentagon(q, base_radius * scale);
            d = smin(d, penta, 0.15);
        }
    }
    
    // Apply secondary grammar deformations
    d = secondaryPattern(p, d, deform_factor);
    
    return d;
}

// Add pattern influence to normal calculation
vec3 calcNormal(vec2 p) {
    const float eps = 0.001;
    vec2 d = vec2(eps, 0);
    
    // Calculate basic SDF for height
    float basic_d = sdf(p);
    
    // Include pattern influence in normal calculation
    vec3 p3 = to3D(p, basic_d);
    
    // Apply same rotation as in SDF
    float slowTime = time * 0.05;
    mat3 rot = rotateY(slowTime) * 
               rotateX(sin(slowTime * 0.5) * 0.05) * 
               rotateZ(cos(slowTime * 0.3) * 0.05);
    
    p3 = rot * p3;
    
    // Calculate normal with pattern influence
    vec3 normal = normalize(vec3(
        sdf(p + d.xy) - sdf(p - d.xy),
        sdf(p + d.yx) - sdf(p - d.yx),
        2.0 * eps
    ));
    
    return normalize(rot * normal);
}

void main() {
    vec2 pos = v_position;
    float d = sdf(pos);
    
    if (d > 0.01) {
        discard;  // Instead of returning transparent black
        return;
    }

    // Normal and view vectors
    vec3 N = calcNormal(pos);
    vec3 V = normalize(vec3(0.0, 0.0, 1.0));

    // Main light setup
    // Update light position to match 3D rotation
    float slowTime = time * 0.1;
    vec3 baseLight = normalize(vec3(0.5, 0.5, 1.0));
    vec3 mainLight = rotateY(slowTime * 0.3) * 
                    rotateX(sin(slowTime * 0.2) * 0.1) * 
                    rotateZ(cos(slowTime * 0.15) * 0.1) * baseLight;
    vec3 rimLight = normalize(vec3(-0.5, -0.5, 0.8));
    
    // Material properties
    vec3 albedo = base_color;
    float ao = smoothstep(-0.2, 0.2, d);
    
    // Main light calculations
    float NdotL = max(dot(N, mainLight), 0.0);
    float NdotV = max(dot(N, V), 0.0);
    vec3 H = normalize(mainLight + V);
    float NdotH = max(dot(N, H), 0.0);
    
    // Rim light
    float rimPower = 3.0;
    float rim = pow(1.0 - NdotV, rimPower) * pow(max(dot(N, rimLight), 0.0), 0.3);
    
    // Specular calculation
    float specPower = mix(32.0, 128.0, 1.0 - roughness);
    float spec = pow(NdotH, specPower);
    vec3 specColor = mix(vec3(0.04), albedo, metallic);
    
    // Fresnel
    float fresnel = pow(1.0 - NdotV, 5.0) * metallic;
    
    // Lighting composition
    vec3 finalColor = vec3(0.0);
    // Ambient
    finalColor += albedo * 0.2 * ao;
    // Diffuse
    finalColor += albedo * NdotL * (1.0 - metallic);
    // Specular
    finalColor += spec * specColor * (0.5 + metallic * 2.0);
    // Rim light
    finalColor += rim * mix(vec3(0.3, 0.4, 0.5), albedo, 0.5);
    // Fresnel
    finalColor += fresnel * specColor;

    // Tone mapping and gamma correction
    finalColor = finalColor / (finalColor + vec3(1.0));
    finalColor = pow(finalColor, vec3(1.0/2.2));
    
    color = vec4(finalColor, 1.0);
}