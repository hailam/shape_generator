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
vec3 lighting(vec3 p,vec3 n,vec3 rd){
    ShapeVariations v=getVariations();
    
    vec3 light_pos=vec3(2.,4.,-3.);
    vec3 light_color=vec3(1.,.95,.8);
    
    vec3 view_dir=-rd;
    vec3 light_dir=normalize(light_pos-p);
    vec3 half_dir=normalize(light_dir+view_dir);
    
    float diff=max(dot(n,light_dir),0.);
    float spec=pow(max(dot(n,half_dir),0.),32.);
    
    vec3 ambient=base_color*.2;
    vec3 diffuse=base_color*light_color*diff;
    vec3 specular=light_color*spec*(1.-v.roughness);
    
    return ambient+diffuse+specular;
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
    
    if(d>MAX_DIST-EPSILON){
        color = vec4(0.0, 0.0, 0.0, 0.0);
    }else{
        vec3 p=ro+rd*d;
        vec3 n=getNormal(p);
        vec3 col=lighting(p,n,rd);
        
        // Apply fog
        float fog=1.-exp(-d*.1);
        col=mix(col,vec3(.1),fog);
        
        // Apply gamma correction
        col=pow(col,vec3(1./2.2));
        
        color=vec4(col,0.0);
    }
}