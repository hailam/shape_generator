#version 140

in vec2 position;
out vec2 v_position;

uniform float view_scale;
uniform vec2 view_offset;

void main() {
    v_position = position;
    vec2 scaled_pos = position * view_scale;
    vec2 final_pos = scaled_pos + view_offset;
    gl_Position = vec4(final_pos, 0.0, 1.0);
}