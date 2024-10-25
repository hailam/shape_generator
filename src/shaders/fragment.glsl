 #version 140

in vec3 v_normal;
in vec3 v_color;
in vec3 v_position;
in vec2 v_tex_coord;
flat in float v_pen_type;

out vec4 color;

float sdSphere(vec3 p, float r) {
    return length(p) - r;
}

float sdBox(vec3 p, vec3 b) {
    vec3 d = abs(p) - b;
    return min(max(d.x, max(d.y, d.z)), 0.0) + length(max(d, 0.0));
}

float getPenEffect(vec3 p, float pen_type) {
    // Simplified pen effects based on type
    float effect = 0.0;
    
    if (pen_type < 1.0) {
        // Spherical effect
        effect = length(p) - 0.5;
    } else if (pen_type < 2.0) {
        // Box effect
        vec3 q = abs(p) - vec3(0.4);
        effect = length(max(q, 0.0)) + min(max(q.x, max(q.y, q.z)), 0.0);
    } else {
        // Fallback to basic sphere
        effect = length(p) - 0.4;
    }
    
    return effect;
}

void main() {
    vec3 light_dir = normalize(vec3(1.0, 1.0, -1.0));
    vec3 normal = normalize(v_normal);
    vec3 view_dir = normalize(-v_position);
    
    // Basic lighting
    float ambient = 0.3;
    float diffuse = max(dot(normal, light_dir), 0.0);
    
    // Pen effect
    vec2 uv = v_tex_coord * 2.0 - 1.0;
    float effect = getPenEffect(vec3(uv, 0.0), v_pen_type);
    float pattern = smoothstep(-0.1, 0.1, sin(effect * 10.0));
    
    // Color modification based on pen type
    vec3 base_color = v_color;
    base_color *= 1.0 + pattern * 0.2;
    
    // Final color
    vec3 final_color = base_color * (ambient + diffuse);
    
    // Rim lighting
    float rim = pow(1.0 - max(dot(normal, view_dir), 0.0), 3.0);
    final_color += vec3(rim * 0.3);
    
    color = vec4(final_color, 1.0);
}