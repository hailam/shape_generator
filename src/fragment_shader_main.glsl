vec3 getTexturePattern(vec3 p, MaterialProperties mat) {
    float pattern = 0.0;
    float scale = mat.textureScale;
    
    // Mix different texture patterns based on material properties
    if(mat.texturePattern < 0.2) {
        // Noise texture
        pattern = fbm(p * scale);
    } else if(mat.texturePattern < 0.4) {
        // Stripes
        pattern = stripePattern(p, scale);
    } else if(mat.texturePattern < 0.6) {
        // Checkers
        pattern = checkersPattern(p, scale);
    } else if(mat.texturePattern < 0.8) {
        // Voronoi cells
        pattern = voronoiNoise(p * scale);
    } else {
        // Hexagon pattern
        pattern = hexagonPattern(p, scale);
    }
    
    return vec3(pattern);
}


// Ray marching
float rayMarch(vec3 ro,vec3 rd){
    float dO=0.;
    
    for(int i=0;i<MAX_STEPS;i++){
        vec3 p=ro+rd*dO;
        float dS=sdEnhancedShape(p,getVariations());
        dO+=dS;
        if(dS<EPSILON||dO>MAX_DIST)break;
    }
    
    return dO;
}

// Calculate normals
vec3 getNormal(vec3 p){
    vec2 e=vec2(EPSILON,0.);
    ShapeVariations v=getVariations();
    return normalize(vec3(
            sdEnhancedShape(p+e.xyy,v)-sdEnhancedShape(p-e.xyy,v),
            sdEnhancedShape(p+e.yxy,v)-sdEnhancedShape(p-e.yxy,v),
            sdEnhancedShape(p+e.yyx,v)-sdEnhancedShape(p-e.yyx,v)
        ));
}
    
// Lighting calculation
vec3 lighting(vec3 p, vec3 n, vec3 rd) {
    ShapeVariations v = getVariations();
    MaterialProperties mat = getMaterialProperties(v);
    
    vec3 light_pos = vec3(2., 4., -3.);
    vec3 light_color = vec3(1., .95, .8);
    
    vec3 view_dir = -rd;
    vec3 light_dir = normalize(light_pos - p);
    vec3 half_dir = normalize(light_dir + view_dir);
    
    // Basic lighting
    float diff = max(dot(n, light_dir), 0.0);
    float spec = pow(max(dot(n, half_dir), 0.0), 32.0) * mat.specularStrength;
    
    // Fresnel/rim lighting
    float rim = pow(1.0 - max(dot(n, view_dir), 0.0), 4.0) * mat.rimLight;
    
    // Get texture pattern
    vec3 pattern = getTexturePattern(p, mat);
    
    // Combine colors
    vec3 ambient = base_color * 0.2;
    vec3 diffuse = base_color * light_color * diff;
    vec3 specular = light_color * spec * (1.0 - mat.roughness);
    vec3 rim_color = light_color * rim;
    
    // Mix with texture pattern
    vec3 color = mix(
        ambient + diffuse + specular + rim_color,
        pattern * diffuse,
        mat.textureMix
    );
    
    // Metallic reflection
    if(mat.metallic > 0.0) {
        vec3 refl = reflect(rd, n);
        vec3 refl_color = vec3(
            fbm(refl * 2.0),
            fbm(refl * 2.1),
            fbm(refl * 2.2)
        );
        color = mix(color, refl_color, mat.metallic);
    }
    
    return color;
}

// Main function
void main(){
    vec2 uv=v_position.xy;
    
    // Camera setup
    float camera_radius=4.;
    float camera_height=2.;
    
    vec3 ro=vec3(
        cos(time*.5)*camera_radius,
        sin(time*.3)*camera_height,
        sin(time*.5)*camera_radius
    );
    
    vec3 lookAt=vec3(0.,0.,0.);
    vec3 forward=normalize(lookAt-ro);
    vec3 right=normalize(cross(vec3(0.,1.,0.),forward));
    vec3 up=cross(forward,right);
    
    vec3 rd=normalize(forward+(uv.x*right+uv.y*up)*.7);
    
    float d=rayMarch(ro,rd);
    
    if(d>MAX_DIST-EPSILON) {
        color = vec4(0.0, 0.0, 0.0, 0.0);
    } else {
        vec3 p = ro + rd * d;
        vec3 n = getNormal(p);
        vec3 col = lighting(p, n, rd);
        
        // Make alpha dependent on the material
        float alpha = 1.0;
        MaterialProperties mat = getMaterialProperties(getVariations());
        if(mat.metallic > 0.5) {
            alpha = 0.8; // Make metallic materials slightly transparent
        }
        
        // Apply gamma correction
        col = pow(col, vec3(1./2.2));
        
        color = vec4(col, alpha);
    }

}