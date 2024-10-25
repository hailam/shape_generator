#version 140

in vec3 position;
in vec3 normal;
in vec3 color;
in vec2 texture_coord;
in uint pen_type;

out vec3 v_normal;
out vec3 v_color;
out vec3 v_position;
out vec2 v_tex_coord;
flat out float v_pen_type;  // Change to float for compatibility

uniform mat4 matrix;

void main() {
    v_normal = normal;
    v_color = color;
    v_position = position;
    v_tex_coord = texture_coord;
    v_pen_type = float(pen_type);  // Convert to float
    gl_Position = matrix * vec4(position, 1.0);
}