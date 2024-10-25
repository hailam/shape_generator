// vertex.glsl
#version 140

in vec2 position;
out vec3 v_position;

void main() {
    v_position = vec3(position, 1.0);
    gl_Position = vec4(position, 0.0, 1.0);
}