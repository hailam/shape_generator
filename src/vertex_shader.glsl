#version 140

in vec3 position;
in vec3 normal;
in vec3 color;

out vec3 v_normal;
out vec3 v_position;
out vec3 v_color;

uniform mat4 perspective_matrix;
uniform mat4 view_matrix;
uniform mat4 model_matrix;

void main() {
    mat4 modelview = view_matrix * model_matrix;
    // Instead of using inverse matrix, we'll transform the normal directly
    // This is simpler but works well enough for basic rendering
    v_normal = mat3(modelview) * normal;
    
    vec4 world_pos = model_matrix * vec4(position, 1.0);
    v_position = world_pos.xyz;
    v_color = color;
    
    gl_Position = perspective_matrix * view_matrix * world_pos;
}