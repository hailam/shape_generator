#version 140

in vec3 v_normal;
in vec3 v_position;
in vec3 v_color;

out vec4 color;

uniform vec3 light_position;

void main() {
    vec3 normal = normalize(v_normal);
    vec3 light_dir = normalize(light_position - v_position);
    
    // Ambient lighting
    float ambient_strength = 0.2;
    vec3 ambient = ambient_strength * v_color;
    
    // Diffuse lighting
    float diff = max(dot(normal, light_dir), 0.0);
    vec3 diffuse = diff * v_color;
    
    // Specular lighting
    float specular_strength = 0.5;
    vec3 view_dir = normalize(-v_position);
    vec3 reflect_dir = reflect(-light_dir, normal);
    float spec = pow(max(dot(view_dir, reflect_dir), 0.0), 32);
    vec3 specular = specular_strength * spec * vec3(1.0, 1.0, 1.0);
    
    // Rim lighting
    float rim_strength = 0.3;
    float rim = 1.0 - max(dot(view_dir, normal), 0.0);
    rim = pow(rim, 3);
    vec3 rim_light = rim_strength * rim * v_color;
    
    vec3 result = ambient + diffuse + specular + rim_light;
    color = vec4(result, 1.0);
}