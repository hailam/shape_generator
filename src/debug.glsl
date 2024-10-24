#version 140

in vec3 v_position;
out vec4 color;

// Global constants
const float MAX_DIST = 100.0;
const float EPSILON = 0.001;
const int MAX_STEPS = 100;
const float PI = 3.14159265359;
const float FIXED_SCALE = 1.0;

// Uniforms
uniform float shape_type;
uniform float scale;
uniform float rotation;
uniform vec3 base_color;
uniform float time;

// Variation buffers
layout(std140) uniform VariationsBuffer1 {
    float variations1[16];
};
layout(std140) uniform VariationsBuffer2 {
    float variations2[16];
};
layout(std140) uniform VariationsBuffer3 {
    float variations3[8];
};

float getVariation(int index) {
    if(index < 16) {
        return variations1[index];
    } else if(index < 32) {
        return variations2[index-16];
    } else {
        return variations3[index-32];
    }
}

// Matrix operations
mat2 rot2D(float angle) {
    float s = sin(angle);
    float c = cos(angle);
    return mat2(c, -s, s, c);
}

mat3 rotateY(float angle) {
    float s = sin(angle);
    float c = cos(angle);
    return mat3(
        c, 0.0, -s,
        0.0, 1.0, 0.0,
        s, 0.0, c
    );
}

mat3 rotateX(float angle) {
    float s = sin(angle);
    float c = cos(angle);
    return mat3(
        1.0, 0.0, 0.0,
        0.0, c, -s,
        0.0, s, c
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

// Operations
float opSmoothUnion(float d1, float d2, float k) {
    float h = clamp(0.5 + 0.5*(d2-d1)/k, 0.0, 1.0);
    return mix(d2, d1, h) - k*h*(1.0-h);
}

float opSmoothSubtraction(float d1, float d2, float k) {
    float h = clamp(0.5 - 0.5*(d2+d1)/k, 0.0, 1.0);
    return mix(d2, -d1, h) + k*h*(1.0-h);
}

float opSmoothIntersection(float d1, float d2, float k) {
    float h = clamp(0.5 - 0.5*(d2-d1)/k, 0.0, 1.0);
    return mix(d2, d1, h) + k*h*(1.0-h);
}

// Basic SDFs
float sdSphere(vec3 p, float r) {
    return length(p) - r;
}

float sdBox(vec3 p, vec3 b) {
    vec3 q = abs(p) - b;
    return length(max(q, 0.0)) + min(max(q.x, max(q.y, q.z)), 0.0);
}

float sdOctahedron(vec3 p, float s) {
    p = abs(p);
    return (p.x + p.y + p.z - s) * 0.57735027;
}

float sdTorus(vec3 p, vec2 t) {
    vec2 q = vec2(length(p.xz) - t.x, p.y);
    return length(q) - t.y;
}

float sdCross(vec3 p, float r) {
    float x = length(p.yz) - r;
    float y = length(p.xz) - r;
    float z = length(p.xy) - r;
    return min(min(x, y), z);
}

float sdPyramid(vec3 p, float h) {
    float m2 = h*h + 0.25;
    p.xz = abs(p.xz);
    p.xz = (p.z>p.x) ? p.zx : p.xz;
    p.xz -= 0.5;
    vec3 q = vec3(p.z, h*p.y - 0.5*p.x, h*p.x + 0.5*p.y);
    float s = max(-q.x, 0.0);
    float t = clamp((q.y-0.5*p.z)/(m2+0.25), 0.0, 1.0);
    float a = m2*(q.x+s)*(q.x+s) + q.y*q.y;
    float b = m2*(q.x+0.5*t)*(q.x+0.5*t) + (q.y-m2*t)*(q.y-m2*t);
    float d2 = min(q.y,-q.x*m2-q.y*0.5) > 0.0 ? 0.0 : min(a,b);
    return sqrt((d2+q.z*q.z)/m2) * sign(max(q.z,-p.y));
}

float sdCylinder(vec3 p, vec2 h) {
    vec2 d = abs(vec2(length(p.xz), p.y)) - h;
    return min(max(d.x, d.y), 0.0) + length(max(d, 0.0));
}

float sdScene(vec3 p) {
    p *= FIXED_SCALE;
    p = rotateY(rotation) * p;
    
    float param1 = getVariation(0);
    float param2 = getVariation(1);
    float param3 = getVariation(2);
    float param4 = getVariation(3);
    
    float timeScale = 0.15;
    float mainSize = 0.35;
    float shape;
    
    // First pass: Core Structure
    if(shape_type >= 10.0) {
        // Cubic lattice with spherical nodes
        vec3 p1 = p;
        p1 = rotateY(time * timeScale) * p1;
        
        float cube = sdBox(p1, vec3(mainSize));
        float spheres = MAX_DIST;
        
        for(int i = 0; i < 8; i++) {
            vec3 offset = vec3(
                float(i & 1) * 2.0 - 1.0,
                float((i >> 1) & 1) * 2.0 - 1.0,
                float((i >> 2) & 1) * 2.0 - 1.0
            ) * mainSize;
            spheres = opSmoothUnion(
                spheres,
                sdSphere(p1 - offset * 0.5, mainSize * 0.2),
                0.1
            );
        }
        shape = opSmoothUnion(cube, spheres, 0.2);
    }
    else if(shape_type >= 20.0) {
        // Platonic solid with cuts
        vec3 p1 = p;
        float solid = sdOctahedron(p1, mainSize);
        
        for(int i = 0; i < 3; i++) {
            p1 = rotateY(PI / 3.0 * float(i)) * p1;
            float cut = sdBox(p1, vec3(mainSize * 2.0, mainSize * 0.1, mainSize * 2.0));
            solid = opSmoothSubtraction(cut, solid, 0.05);
        }
        shape = solid;
    }
    else if (shape_type >= 30.0) {
        // Interlocked rings
        vec3 p1 = p;
        shape = MAX_DIST;
        
        vec3 scales = vec3(
            1.0 + sin(time * timeScale) * 0.2,
            1.0 + cos(time * timeScale * 1.1) * 0.2,
            1.0 + sin(time * timeScale * 0.9) * 0.2
        );
        
        float ring1 = sdTorus(p1 * scales.x, vec2(mainSize, mainSize * 0.15));
        float ring2 = sdTorus(rotateY(PI/2.0) * (p1 * scales.y), vec2(mainSize, mainSize * 0.15));
        float ring3 = sdTorus(rotateX(PI/2.0) * (p1 * scales.z), vec2(mainSize, mainSize * 0.15));
        
        shape = opSmoothUnion(ring1, ring2, 0.1);
        shape = opSmoothUnion(shape, ring3, 0.1);
    }
    else {
        // Crystal cluster
        vec3 p1 = p;
        shape = MAX_DIST;
        
        for(int i = 0; i < 5; i++) {
            float angle = float(i) * PI * 2.0/5.0;
            vec3 offset = vec3(
                cos(angle) * mainSize * 0.7,
                (float(i % 2) - 0.5) * mainSize,
                sin(angle) * mainSize * 0.7
            );
            
            vec3 p2 = rotateY(angle + time * timeScale) * (p1 - offset);
            float crystal = sdOctahedron(p2, mainSize * 0.3);
            shape = opSmoothUnion(shape, crystal, 0.15);
        }
    }
    
    // Second pass: Structure Modification
    if(param2 < 0.33) {
        // Geometric intersection
        vec3 p2 = rotateY(time * timeScale) * p;
        float intersect = sdBox(p2, vec3(mainSize * 0.8));
        shape = opSmoothIntersection(shape, intersect, 0.1);
    }
    else if(param2 < 0.66) {
        // Fractal cuts
        vec3 p2 = p * 3.0;
        float cuts = abs(sin(p2.x) * sin(p2.y) * sin(p2.z)) - 0.2;
        shape = opSmoothSubtraction(cuts, shape, 0.1);
    }
    else {
        // Voronoi displacement
        vec3 p2 = floor(p * 5.0);
        float displacement = (
            sin(p2.x * 12.3 + time) +
            sin(p2.y * 15.7) +
            sin(p2.z * 17.1)
        ) * mainSize * 0.1;
        shape += displacement;
    }
    
    // Third pass: Detail Addition
    if(param3 < 0.33) {
        // Angular facets
        vec3 p3 = p * 8.0;
        float facets = max(
            abs(fract(p3.x + p3.y) - 0.5),
            abs(fract(p3.y + p3.z) - 0.5)
        ) * mainSize * 0.1;
        shape -= facets;
    }
    else if(param3 < 0.66) {
        // Layered patterns
        float layers = sin(length(p.xz) * 8.0 + p.y * 6.0) * 
                      cos(length(p.yz) * 8.0 + p.x * 6.0) * mainSize * 0.1;
        shape += layers;
    }
    else {
        // Grid pattern
        vec3 p3 = abs(fract(p * 5.0) - 0.5);
        float grid = length(p3) * mainSize * 0.2;
        shape = opSmoothSubtraction(grid, shape, 0.05);
    }
    
    return shape / FIXED_SCALE;
}

float rayMarch(vec3 ro, vec3 rd) {
    float dO = 0.0;
    for(int i = 0; i < MAX_STEPS; i++) {
        vec3 p = ro + rd * dO;
        float dS = sdScene(p);
        dO += dS;
        if(dS < EPSILON || dO > MAX_DIST) break;
    }
    return dO;
}

vec3 getNormal(vec3 p) {
    vec2 e = vec2(EPSILON, 0.0);
    float d = sdScene(p);
    vec3 n = d - vec3(
        sdScene(p - e.xyy),
        sdScene(p - e.yxy),
        sdScene(p - e.yyx)
    );
    return normalize(n);
}

vec3 lighting(vec3 p, vec3 n, vec3 rd) {
    vec3 light_pos = vec3(2.0, 4.0, -3.0);
    vec3 light_color = vec3(1.0, 0.95, 0.8);
    vec3 view_dir = -rd;
    vec3 light_dir = normalize(light_pos - p);
    vec3 half_dir = normalize(light_dir + view_dir);
    
    float diff = max(dot(n, light_dir), 0.0);
    float spec = pow(max(dot(n, half_dir), 0.0), 32.0);
    
    vec3 ambient = vec3(0.2);
    vec3 diffuse = light_color * diff;
    vec3 specular = light_color * spec * 0.5;
    
    float rim = 1.0 - max(dot(view_dir, n), 0.0);
    rim = pow(rim, 3.0);
    
    return (ambient + diffuse + specular) * base_color + rim * 0.2;
}

void main() {
    vec2 uv = v_position.xy;
    
    float camRadius = 3.0;
    float camSpeed = 0.15;
    vec3 ro = vec3(
        sin(time * camSpeed) * camRadius,
        1.5 + sin(time * camSpeed * 0.5) * 0.2,
        cos(time * camSpeed) * camRadius
    );
    
    vec3 lookAt = vec3(0.0, 0.0, 0.0);
    vec3 forward = normalize(lookAt - ro);
    vec3 right = normalize(cross(vec3(0.0, 1.0, 0.0), forward));
    vec3 up = cross(forward, right);
    
    vec3 rd = normalize(forward + (uv.x * right + uv.y * up) * 0.7);
    
    float d = rayMarch(ro, rd);
    
    if(d > MAX_DIST - EPSILON) {
        color = vec4(0.0, 0.0, 0.0, 0.0);
    } else {
        vec3 p = ro + rd * d;
        vec3 n = getNormal(p);
        vec3 col = lighting(p, n, rd);
        
        float fog = 1.0 - exp(-d * 0.1);
        col = mix(col, vec3(0.1), fog);
        
        col = pow(col, vec3(1.0/2.2));
        
        color = vec4(col, 1.0);
    }
}