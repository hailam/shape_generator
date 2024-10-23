float getVariation(int index){
    if(index<16){
        return variations1[index];
    }else if(index<32){
        return variations2[index-16];
    }else{
        return variations3[index-32];
    }
}

// helper functions for basic operations
float smin(float a,float b,float k){
    float h=max(k-abs(a-b),0.)/k;
    return min(a,b)-h*h*k*(1./4.);
}

mat2 rot2D(float angle){
    float s=sin(angle);
    float c=cos(angle);
    return mat2(c,-s,s,c);
}

vec3 twist(vec3 p,float amount){
    float c=cos(amount*p.y);
    float s=sin(amount*p.y);
    mat2 m=mat2(c,-s,s,c);
    return vec3(m*p.xz,p.y);
}

vec3 bend(vec3 p,float amount){
    float c=cos(amount*p.x);
    float s=sin(amount*p.x);
    mat2 m=mat2(c,-s,s,c);
    return vec3(p.x,m*p.yz);
}
// end of helper functions

// Noise functions
float rand(vec3 p){
    return fract(sin(dot(p,vec3(12.9898,78.233,45.543)))*43758.5453);
}

float noise(vec3 p){
    vec3 i=floor(p);
    vec3 f=fract(p);
    f=f*f*(3.-2.*f);
    
    return mix(
        mix(
            mix(rand(i),rand(i+vec3(1,0,0)),f.x),
            mix(rand(i+vec3(0,1,0)),rand(i+vec3(1,1,0)),f.x),
            f.y
        ),
        mix(
            mix(rand(i+vec3(0,0,1)),rand(i+vec3(1,0,1)),f.x),
            mix(rand(i+vec3(0,1,1)),rand(i+vec3(1,1,1)),f.x),
            f.y
        ),
        f.z
    );
}
// end of noise functions