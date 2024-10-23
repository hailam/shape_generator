#version 140

in vec3 v_position;
out vec4 color;

// Global constants
const float MAX_DIST=100.;
const float EPSILON=.001;
const int MAX_STEPS=100;
const float PI=3.14159265359;

// Basic uniforms
uniform float shape_type;// Now 0-40 range
uniform float scale;
uniform float rotation;
uniform vec3 base_color;
uniform float time;

// Split variations buffer
layout(std140)uniform VariationsBuffer1{
    float variations1[16];
};
layout(std140)uniform VariationsBuffer2{
    float variations2[16];
};
layout(std140)uniform VariationsBuffer3{
    float variations3[8];
};

