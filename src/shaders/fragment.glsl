#version 140

in vec2 v_position;
out vec4 color;

// Base uniforms
uniform float time;
uniform float speed;
uniform int base_shape_type;
uniform float base_radius;
uniform vec3 base_color;
uniform vec3 shape_scale;
uniform vec3 shape_rotation;
uniform float metallic;
uniform float roughness;
uniform float deform_factor;
uniform uint pattern;

// Variation uniforms
uniform vec4 var1_0, var1_1, var1_2, var1_3;
uniform vec4 var2_0, var2_1, var2_2, var2_3;
uniform vec4 var3_0, var3_1, var3_2, var3_3;

// Modifier uniforms
uniform int modifier_count;
uniform int[4] modifier_types;
uniform float[4] modifier_params;

// Light uniforms
uniform vec3 light_pos_0;
uniform vec3 light_pos_1;
uniform vec3 light_pos_2;
uniform vec3 light_pos_3;

uniform vec3 light_color_0;
uniform vec3 light_color_1;
uniform vec3 light_color_2;
uniform vec3 light_color_3;

const float PI = 3.14159265359;
const float PHI = 1.61803398875;
const float TAU = PI * 2.0;

// Helper Functions
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

vec4 getVariation(int buffer, int index) {
    if (buffer == 1) {
        if (index == 0) return var1_0;
        if (index == 1) return var1_1;
        if (index == 2) return var1_2;
        return var1_3;
    } else if (buffer == 2) {
        if (index == 0) return var2_0;
        if (index == 1) return var2_1;
        if (index == 2) return var2_2;
        return var2_3;
    } else {
        if (index == 0) return var3_0;
        if (index == 1) return var3_1;
        if (index == 2) return var3_2;
        return var3_3;
    }
}

// SDF Functions
float sdSphere(vec3 p, float r) {
    return length(p) - r;
}

float sdBox(vec3 p, vec3 b) {
    vec3 d = abs(p) - b;
    return min(max(d.x, max(d.y, d.z)), 0.0) + length(max(d, 0.0));
}

float sdCylinder(vec3 p, vec2 h) {
    vec2 d = abs(vec2(length(p.xz), p.y)) - h;
    return min(max(d.x, d.y), 0.0) + length(max(d, 0.0));
}

// Deformation Functions
vec3 twist(vec3 p, float amount) {
    float c = cos(amount * p.y);
    float s = sin(amount * p.y);
    mat2 m = mat2(c, -s, s, c);
    return vec3(m * p.xz, p.y);
}

vec3 bend(vec3 p, float amount) {
    float c = cos(amount * p.x);
    float s = sin(amount * p.x);
    mat2 m = mat2(c, -s, s, c);
    return vec3(p.x, m * p.yz);
}

// flatten like a pancake
vec3 flatten(vec3 p, float amount) {
    float c = cos(amount * p.z);
    float s = sin(amount * p.z);
    mat2 m = mat2(c, -s, s, c);
    return vec3(m * p.xy, p.z);
}

float applyVariations(vec3 p, float d) {
    // Primary variations
    for(int i = 0; i < 4; i++) {
        vec4 var = getVariation(1, i);
        float freq = var.x;
        float amp = var.y;
        vec3 dir = normalize(vec3(var.z, var.w, var.x));
        d += sin(dot(p, dir) * freq) * amp;
    }
    
    // Secondary variations
    for(int i = 0; i < 4; i++) {
        vec4 var = getVariation(2, i);
        float freq = var.x * 2.0;
        float amp = var.y * 0.5;
        vec3 dir = normalize(vec3(var.z, var.w, var.x));
        d += sin(dot(p, dir) * freq) * amp * smoothstep(0.0, 0.1, -d);
    }
    
    // Tertiary variations
    for(int i = 0; i < 4; i++) {
        vec4 var = getVariation(3, i);
        float freq = var.x * 4.0;
        float amp = var.y * 0.25;
        vec3 dir = normalize(vec3(var.z, var.w, var.x));
        d += sin(dot(p, dir) * freq) * amp * smoothstep(0.0, 0.05, -d);
    }
    
    return d;
}

float sdf(vec3 p) {
    // Apply time-based animation
    float slowTime = time * speed * 0.1;
    p = rotateY(slowTime) * rotateX(sin(slowTime * 0.5)) * rotateZ(cos(slowTime * 0.3)) * p;
    
    // Apply shape scale and rotation
    p = rotateX(shape_rotation.x) * rotateY(shape_rotation.y) * rotateZ(shape_rotation.z) * p;
    p /= shape_scale;
    
    // Base shape
    float d = 1e10;
    if (base_shape_type == 0) {
        d = sdSphere(p, base_radius);
    } else if (base_shape_type == 1) {
        d = sdBox(p, vec3(base_radius));
    } else {
        d = sdCylinder(p, vec2(base_radius, base_radius * 2.0));
    }
    
    // Apply modifiers
    for(int i = 0; i < modifier_count && i < 4; i++) {
        if (modifier_types[i] == 0) { // Twist
            p = twist(p, modifier_params[i]);
        } else if (modifier_types[i] == 1) { // Bend
            p = bend(p, modifier_params[i]);
        }
    }

    p = twist(p, 5.0);
    p = bend(p, 3.0);
    p = flatten(p, 1.0);
    
    // Apply variations
    d = applyVariations(p, d);
    
    // Scale back
    d *= min(shape_scale.x, min(shape_scale.y, shape_scale.z));

    
    
    return d;
}

vec3 calcNormal(vec3 p) {
    const float eps = 0.001;
    const vec2 h = vec2(eps, 0);
    return normalize(vec3(
        sdf(p + h.xyy) - sdf(p - h.xyy),
        sdf(p + h.yxy) - sdf(p - h.yxy),
        sdf(p + h.yyx) - sdf(p - h.yyx)
    ));
}

// Function to calculate light contribution
vec3 calcLight(vec3 lightPos, vec3 lightColor, vec3 hitPos, vec3 N, vec3 V) {
    vec3 L = normalize(lightPos - hitPos);
    vec3 H = normalize(L + V);
    
    float NdotL = max(dot(N, L), 0.0);
    float NdotH = max(dot(N, H), 0.0);
    
    // Distance attenuation
    float dist = length(lightPos - hitPos);
    float attenuation = 1.0 / (1.0 + 0.1 * dist + 0.01 * dist * dist);
    
    // Specular
    float specPower = mix(32.0, 128.0, 1.0 - roughness);
    float spec = pow(NdotH, specPower);
    
    // Fresnel
    float fresnel = pow(1.0 - max(dot(N, V), 0.0), 5.0) * metallic;
    
    // Combine components
    vec3 diffuseContrib = base_color * NdotL * (1.0 - metallic);
    vec3 specularContrib = spec * mix(vec3(0.04), base_color, metallic) * (0.5 + metallic * 2.0);
    
    return (diffuseContrib + specularContrib) * lightColor * attenuation;
}

void main() {
     // Camera setup
    vec3 cameraPos = vec3(0.0, 0.0, 0.8);
    vec3 cameraTarget = vec3(0.0, 0.0, 0.0);
    vec3 cameraUp = vec3(0.0, 1.0, 0.0);
    
    // Calculate ray direction
    vec3 rayDir = normalize(vec3(v_position * 2.0, -1.0));
    vec3 pos = cameraPos;
    
    // Ray marching
    float t = 0.0;
    float d = 0.0;
    for(int i = 0; i < 64; i++) {
        pos = cameraPos + rayDir * t;
        d = sdf(pos);
        if(abs(d) < 0.001 || t > 10.0) break;
        t += d;
    }
    
    if(t > 10.0) {
        discard;
        return;
    }

    vec3 hitPos = cameraPos + rayDir * t;
    vec3 N = calcNormal(hitPos);
    vec3 V = normalize(cameraPos - hitPos);

    // Initialize lighting components
    vec3 ambient = vec3(0.0);
    vec3 diffuse = vec3(0.0);
    vec3 specular = vec3(0.0);
    
    // Key light (main light)
    vec3 mainLightPos = vec3(2.0, 2.0, 4.0);
    vec3 mainLightColor = vec3(1.0, 0.9, 0.8) * 1.0;
    
    // Fill light (softer, from opposite side)
    vec3 fillLightPos = vec3(-3.0, 1.0, 2.0);
    vec3 fillLightColor = vec3(0.4, 0.5, 0.6) * 0.5;
    
    // Rim light (from behind)
    vec3 rimLightPos = vec3(0.0, 2.0, -3.0);
    vec3 rimLightColor = vec3(0.5, 0.5, 0.6) * 0.3;
    
    // Top light (for better detail visibility)
    vec3 topLightPos = vec3(0.0, 4.0, 0.0);
    vec3 topLightColor = vec3(0.6, 0.6, 0.6) * 0.4;

    // Calculate each light's contribution
    vec3 finalColor = vec3(0.0);

    // Calculate each light's contribution
    finalColor += calcLight(light_pos_0, light_color_0, hitPos, N, V);
    finalColor += calcLight(light_pos_1, light_color_1, hitPos, N, V);
    finalColor += calcLight(light_pos_2, light_color_2, hitPos, N, V);
    finalColor += calcLight(light_pos_3, light_color_3, hitPos, N, V);
    
    // Ambient occlusion
    float ao = smoothstep(-0.2, 0.2, d);
    vec3 ambientLight = base_color * 0.2 * ao;
    finalColor += ambientLight;
    
    // Edge highlighting
    //float edgeFactor = 1.0 - pow(max(dot(N, V), 0.0), 2.0);
    //finalColor += edgeFactor * base_color * 0.2;
    
    // Surface detail enhancement
    float detailEnhancement = pow(abs(sin(d * 50.0)), 0.5) * 0.1;
    finalColor += detailEnhancement * base_color;

    // HDR tonemapping
    //finalColor = finalColor / (finalColor + vec3(1.0));
    
    // Contrast enhancement
    finalColor = pow(finalColor, vec3(0.9));
    
    // Gamma correction
    //finalColor = pow(finalColor, vec3(1.0/2.2));
    
    color = vec4(finalColor, 1.0);
}