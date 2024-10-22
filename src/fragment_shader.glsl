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

struct ShapeVariations{
    float size_x;// variations[0]
    float size_y;// variations[1]
    float size_z;// variations[2]
    float twist_amount;// variations[3]
    float bend_amount;// variations[4]
    float taper_amount;// variations[5]
    float skew_x;// variations[6]
    float skew_y;// variations[7]
    float skew_z;// variations[8]
    float roundness;// variations[9]
    float bevel_amount;// variations[10]
    float smooth_factor;// variations[11]
    float segments;// variations[12]
    float repeat_x;// variations[13]
    float repeat_y;// variations[14]
    float repeat_z;// variations[15]
    float phase_shift;// variations[16]
    float noise_scale;// variations[17]
    float noise_amount;// variations[18]
    float ripple_freq;// variations[19]
    float ripple_amp;// variations[20]
    float blend_factor;// variations[21]
    float cut_amount;// variations[22]
    float hollow_factor;// variations[23]
    float wobble_speed;// variations[24]
    float pulse_amount;// variations[25]
    float wave_freq;// variations[26]
    float metallic;// variations[27]
    float roughness;// variations[28]
    float emission;// variations[29]
    float extra_param1;// variations[30]
    float extra_param2;// variations[31]
    float extra_param3;// variations[32]
    float extra_param4;// variations[33]
};

ShapeVariations getVariations(){
    ShapeVariations vars;
    vars.size_x=max(.5+getVariation(0),.1);
    vars.size_y=max(.5+getVariation(1),.1);
    vars.size_z=max(.5+getVariation(2),.1);
    vars.twist_amount=getVariation(3)*3.;
    vars.bend_amount=getVariation(4)*2.;
    vars.taper_amount=getVariation(5)*.5;
    vars.skew_x=getVariation(6)*2.-1.;
    vars.skew_y=getVariation(7)*2.-1.;
    vars.skew_z=getVariation(8)*2.-1.;
    vars.roundness=getVariation(9)*.3;
    vars.bevel_amount=getVariation(10)*.2;
    vars.smooth_factor=getVariation(11)*.5;
    vars.segments=floor(2.+getVariation(12)*8.);
    vars.repeat_x=floor(1.+getVariation(13)*2.);
    vars.repeat_y=floor(1.+getVariation(14)*2.);
    vars.repeat_z=floor(1.+getVariation(15)*2.);
    vars.phase_shift=getVariation(16)*PI*2.;
    vars.noise_scale=1.+getVariation(17)*3.;
    vars.noise_amount=getVariation(18)*.3;
    vars.ripple_freq=1.+getVariation(19)*5.;
    vars.ripple_amp=getVariation(20)*.2;
    vars.blend_factor=getVariation(21);
    vars.cut_amount=getVariation(22)*.5;
    vars.hollow_factor=getVariation(23)*.5;
    vars.wobble_speed=max(getVariation(24),.1)*2.;
    vars.pulse_amount=getVariation(25)*.2;
    vars.wave_freq=getVariation(26)*3.;
    vars.metallic=getVariation(27);
    vars.roughness=getVariation(28);
    vars.emission=getVariation(29);
    vars.extra_param1=getVariation(30);
    vars.extra_param2=getVariation(31);
    vars.extra_param3=getVariation(32);
    vars.extra_param4=getVariation(33);
    return vars;
}

// Start of SDF primitives
float sdSphere(vec3 p,float r){
    return length(p)-r;
}

float sdBox(vec3 p,vec3 b){
    vec3 q=abs(p)-b;
    return length(max(q,0.))+min(max(q.x,max(q.y,q.z)),0.);
}

float sdRoundBox(vec3 p,vec3 b,float r){
    vec3 q=abs(p)-b;
    return length(max(q,0.))+min(max(q.x,max(q.y,q.z)),0.)-r;
}

float sdTorus(vec3 p,vec2 t){
    vec2 q=vec2(length(p.xz)-t.x,p.y);
    return length(q)-t.y;
}

float sdCylinder(vec3 p,vec2 h){
    vec2 d=abs(vec2(length(p.xz),p.y))-h;
    return min(max(d.x,d.y),0.)+length(max(d,0.));
}

float sdCone(vec3 p,vec2 c,float h){
    float q=length(p.xz);
    return max(dot(c.xy,vec2(q,p.y)),-h-p.y);
}

float sdCapsule(vec3 p,vec3 a,vec3 b,float r){
    vec3 pa=p-a,ba=b-a;
    float h=clamp(dot(pa,ba)/dot(ba,ba),0.,1.);
    return length(pa-ba*h)-r;
}

float sdVerticalCapsule(vec3 p,float h,float r){
    p.y-=clamp(p.y,0.,h);
    return length(p)-r;
}

float sdOctahedron(vec3 p,float s){
    p=abs(p);
    float m=p.x+p.y+p.z-s;
    vec3 q;
    if(3.*p.x<m)q=p.xyz;
    else if(3.*p.y<m)q=p.yzx;
    else if(3.*p.z<m)q=p.zxy;
    else return m*.57735027;
    float k=clamp(.5*(q.z-q.y+s),0.,s);
    return length(vec3(q.x,q.y-s+k,q.z-k));
}

float sdPyramid(vec3 p,float h){
    float m2=h*h+.25;
    p.xz=abs(p.xz);
    p.xz=(p.z>p.x)?p.zx:p.xz;
    p.xz-=.5;
    vec3 q=vec3(p.z,h*p.y-.5*p.x,h*p.x+.5*p.y);
    float s=max(-q.x,0.);
    float t=clamp((q.y-.5*p.z)/(m2+.25),0.,1.);
    float a=m2*(q.x+s)*(q.x+s)+q.y*q.y;
    float b=m2*(q.x+.5*t)*(q.x+.5*t)+(q.y-m2*t)*(q.y-m2*t);
    float d2=min(q.y,-q.x*m2-q.y*.5)>0.?0.:min(a,b);
    return sqrt((d2+q.z*q.z)/m2)*sign(max(q.z,-p.y));
}

float sdTriPrism(vec3 p,vec2 h){
    vec3 q=abs(p);
    return max(q.z-h.y,max(q.x*.866025+p.y*.5,-p.y)-h.x*.5);
}

float sdRhombicDodecahedron(vec3 p,float r){
    p=abs(p);
    float v1=dot(p,normalize(vec3(1.,1.,1.)));
    float v2=dot(p,normalize(vec3(-1.,1.,1.)));
    float v3=dot(p,normalize(vec3(1.,-1.,1.)));
    float v4=dot(p,normalize(vec3(1.,1.,-1.)));
    return max(max(max(v1,v2),v3),v4)-r;
}

float sdGyroid(vec3 p,float scale,float thickness){
    p*=scale;
    float d=dot(sin(p),cos(p.zxy));
    return abs(d)/scale-thickness;
}

float sdTwist(vec3 p,float k){
    float c=cos(k*p.y);
    float s=sin(k*p.y);
    mat2 m=mat2(c,-s,s,c);
    vec3 q=vec3(m*p.xz,p.y);
    return sdBox(q,vec3(.5,1.,.5));
}

float sdCutHollowSphere(vec3 p,float r,float h,float t){
    vec2 q=vec2(length(p.xz),p.y);
    float w=sqrt(r*r-h*h);
    return max(abs(length(q)-r)-t,
    max(q.y-h,w-length(q)));
}

float sdBoundingSphere(vec3 p,float r){
    return length(p)-r;
}

float sdLink(vec3 p,float le,float r1,float r2){
    vec3 q=vec3(p.x,max(abs(p.y)-le,0.),p.z);
    return length(vec2(length(q.xy)-r1,q.z))-r2;
}

float sdStar(vec3 p,float r,float h,int n){
    float an=3.141593/float(n);
    float en=3.141593/2.;
    vec2 acs=vec2(cos(an),sin(an));
    vec2 ecs=vec2(cos(en),sin(en));
    float bn=mod(atan(p.x,p.z),2.*an)-an;
    p.xz=length(p.xz)*vec2(cos(bn),abs(sin(bn)));
    p.xz=p.xz*acs.x-abs(p.xz.yx)*acs.y*vec2(1.,-1.);
    p.xz*=ecs;
    vec2 d=abs(vec2(length(p.xz),p.y))-vec2(r,h);
    return min(max(d.x,d.y),0.)+length(max(d,0.));
}

float sdFractal(vec3 p){
    float d=sdBox(p,vec3(1.));
    float s=1.;
    for(int i=0;i<3;i++){
        vec3 a=mod(p*s,2.)-1.;
        s*=3.;
        vec3 r=abs(1.-3.*abs(a));
        float da=max(r.x,r.y);
        float db=max(r.y,r.z);
        float dc=max(r.z,r.x);
        float c=(min(da,min(db,dc))-1.)/s;
        d=max(d,c);
    }
    return d;
}

float sdWaves(vec3 p,float freq,float amp){
    return p.y-amp*sin(freq*p.x)*cos(freq*p.z);
}

float sdHelix(vec3 p,float radius,float thickness,float pitch){
    float angle=atan(p.z,p.x);
    float r=length(p.xz);
    float y=p.y-pitch*angle/(2.*PI);
    y=mod(y,pitch)-pitch*.5;
    vec2 q=vec2(r-radius,y);
    return length(q)-thickness;
}

float sdGear(vec3 p,float r,float h,float teeth){
    vec2 q=vec2(length(p.xz),p.y);
    float angle=atan(p.z,p.x);
    float d=length(q-vec2(r+sin(angle*teeth)*.1,0.))-h;
    return max(d,abs(p.y)-.2);
}

float sdCellular(vec3 p,float scale,float thickness){
    p*=scale;
    vec3 i=floor(p);
    vec3 f=fract(p);
    
    float min_dist=1.;
    for(int x=-1;x<=1;x++){
        for(int y=-1;y<=1;y++){
            for(int z=-1;z<=1;z++){
                vec3 offset=vec3(x,y,z);
                vec3 r=offset-f+rand(i+offset);
                float d=length(r);
                min_dist=min(min_dist,d);
            }
        }
    }
    return min_dist/scale-thickness;
}

float sdLattice(vec3 p,float thickness){
    vec3 q=abs(fract(p+.5)-.5);// Corrected to use fract for a periodic lattice
    float d1=length(vec2(q.x,q.y))-thickness;
    float d2=length(vec2(q.y,q.z))-thickness;
    float d3=length(vec2(q.z,q.x))-thickness;
    return min(min(d1,d2),d3);
}

float sdTwistedColumn(vec3 p,float radius,float twist){
    float angle=p.y*twist;
    vec2 q=rot2D(angle)*p.xz;
    return length(q)-radius;
}

float sdRecursiveTetrahedron(vec3 p,float scale,int iterations){
    float d=sdOctahedron(p,1.);
    float s=1.;
    for(int i=0;i<iterations;i++){
        vec3 a=mod(p*s,2.)-1.;
        s*=scale;
        vec3 r=abs(1.-3.*abs(a));
        float c=(min(min(r.x,r.y),r.z)-1.)/s;
        d=max(d,c);
    }
    return d;
}

float sdFlower(vec3 p,float radius,float petals){
    float angle=atan(p.z,p.x);
    float r=length(p.xz);
    float petal=abs(sin(angle*petals))*.2;
    return length(vec2(r-radius-petal,p.y))-.1;
}

float sdStackedBoxes(vec3 p,float spacing,float size){
    vec3 q=p;
    q.y=mod(q.y+spacing/2.,spacing)-spacing/2.;
    return sdBox(q,vec3(size));
}

float sdSwirlBox(vec3 p,float radius,float height){
    float angle=atan(p.z,p.x);
    p.xz=rot2D(angle+p.y)*p.xz;
    return sdBox(p,vec3(radius,height,radius));
}

float sdFractalPyramid(vec3 p,float scale,int iterations){
    float d=sdPyramid(p,1.);
    vec3 q=p;
    float s=1.;
    for(int i=0;i<iterations;i++){
        q=abs(q);
        q.y-=.5;
        q*=scale;
        s*=scale;
        float pd=sdPyramid(q,1.)/s;
        d=smin(d,pd,.1);
    }
    return d;
}

float sdInfinitySymbol(vec3 p,float radius,float thickness){
    p.xz=vec2(length(p.xz)-radius,atan(p.z,p.x));
    p.xz=abs(p.xz);
    float d=length(p.xz);
    return length(vec2(d-.5,p.y))-thickness;
}

float sdTorusKnot(vec3 p,float radius,float thickness,float freq1,float freq2){
    float r=length(p.xz);
    float angle=atan(p.z,p.x);
    
    float phase=angle*freq1+p.y*freq2;
    vec3 q=vec3(
        (radius+cos(phase))*cos(angle),
        sin(phase),
        (radius+cos(phase))*sin(angle)
    );
    
    return length(p-q)-thickness;
}

float sdWavyTorus(vec3 p,float majorRadius,float minorRadius,float waves){
    float angle=atan(p.z,p.x);
    float r=length(p.xz);
    float wave=sin(angle*waves)*.1;
    return length(vec2(r-majorRadius-wave,p.y))-minorRadius;
}

float sdModulatedTorus(vec3 p,float majorRadius,float minorRadius,float freq){
    float angle=atan(p.z,p.x);
    float r=length(p.xz);
    float modulation=sin(angle*freq+time)*.2;
    return length(vec2(r-majorRadius,p.y))-(minorRadius+modulation);
}

float sdTwistedTorus(vec3 p,float majorRadius,float minorRadius,float twist){
    float angle=atan(p.z,p.x);
    p.y+=sin(angle*twist)*.2;
    return length(vec2(length(p.xz)-majorRadius,p.y))-minorRadius;
}

float sdBraid(vec3 p,float radius,float thickness,float freq){
    float angle=atan(p.z,p.x);
    float r=length(p.xz);
    float y=p.y-sin(angle*freq)*.2;
    return length(vec2(r-radius,y))-thickness;
}

float sdChainLink(vec3 p,float radius,float thickness,float length){
    float d1=sdTorus(p,vec2(radius,thickness));
    vec3 q=p;
    q.y=abs(q.y)-length;
    float d2=sdCylinder(q,vec2(thickness,thickness));
    return min(d1,d2);
}

// End of SDF primitives

float sdHexPrism(vec3 p,vec2 h){
    const vec3 k=vec3(-.8660254,.5,.57735);
    p=abs(p);
    p.xy-=2.*min(dot(k.xy,p.xy),0.)*k.xy;
    vec2 d=vec2(
        length(p.xy-vec2(clamp(p.x,-k.z*h.x,k.z*h.x),h.x))*sign(p.y-h.x),
        p.z-h.y
    );
    return min(max(d.x,d.y),0.)+length(max(d,0.));
}

// Enhanced shape selection function
float sdEnhancedShape(vec3 p,ShapeVariations v){
    // Debug shape type
    float debug_shape_type=floor(shape_type);
    
    // Apply global transformations
    p/=max(scale,.1);
    
    // Apply repetition if enabled
    if(v.repeat_x>1.||v.repeat_y>1.||v.repeat_z>1.){
        vec3 c=vec3(2./max(v.repeat_x,1.),
        2./max(v.repeat_y,1.),
        2./max(v.repeat_z,1.));
        p=mod(p+.5*c,c)-.5*c;
    }
    
    // Apply deformations
    float wobble=sin(time*v.wobble_speed+v.phase_shift);
    p=twist(p,v.twist_amount*wobble);
    p=bend(p,v.bend_amount);
    
    // Apply noise distortion
    if(v.noise_amount>0.){
        p+=v.noise_amount*vec3(
            noise(p*v.noise_scale),
            noise(p*v.noise_scale+100.),
            noise(p*v.noise_scale+300.)
        );
    }
    
    // Shape selection
    float d=MAX_DIST;
    float shape_selector=mod(debug_shape_type,40.);
    
    if(shape_selector<1.){
        d=sdSphere(p,1.+v.extra_param1*.2);
    }
    else if(shape_selector<2.){
        d=sdBox(p,vec3(v.size_x,v.size_y,v.size_z));
    }
    else if(shape_selector<3.){
        d=sdRoundBox(p,vec3(v.size_x,v.size_y,v.size_z),v.roundness);
    }
    else if(shape_selector<4.){
        d=sdTorus(p,vec2(1.+v.extra_param1*.3,.3+v.extra_param2*.2));
    }
    else if(shape_selector<5.){
        d=sdCapsule(p,vec3(0,-1,0),vec3(0,1,0),.3+v.extra_param1*.2);
    }
    else if(shape_selector<6.){
        d=sdCone(p,vec2(v.size_x,1.),1.5+v.extra_param1*.5);
    }
    else if(shape_selector<7.){
        d=sdHexPrism(p,vec2(1.+v.extra_param1*.3,.5+v.extra_param2*.3));
    }
    else if(shape_selector<8.){
        d=sdOctahedron(p,1.+v.extra_param1*.3);
    }
    else if(shape_selector<9.){
        d=sdPyramid(p,1.5+v.extra_param1*.5);
    }
    else if(shape_selector<10.){
        float d1=sdSphere(p,1.+v.extra_param1*.3);
        float d2=sdBox(p-vec3(0,sin(time)*.5,0),vec3(.3));
        d=smin(d1,d2,.3);
    }
    else if(shape_selector<11.){
        float angle=atan(p.z,p.x);
        float r=length(p.xz);
        d=max(sdTorus(p,vec2(1.,.3)),sin(angle*8.+time)*.1);
    }
    else if(shape_selector<12.){
        vec3 q=p;
        q.y+=sin(q.x*3.+time)*.2;
        d=sdBox(q,vec3(1.,.2,.2));
    }
    else if(shape_selector<13.){
        float waves=sin(p.x*5.)*cos(p.z*5.)*.2;
        d=sdBox(p-vec3(0,waves,0),vec3(1.,.1,1.));
    }
    else if(shape_selector<14.){
        vec3 q=p;
        float twist=q.y*2.;
        float c=cos(twist+time);
        float s=sin(twist+time);
        q.xz=mat2(c,-s,s,c)*q.xz;
        d=sdBox(q,vec3(.5,1.,.5));
    }
    else if(shape_selector<15.){
        d=max(sdSphere(p,1.),-sdBox(p,vec3(.8)));
    }
    else if(shape_selector<16.){
        float cells=sin(p.x*10.)*sin(p.y*10.)*sin(p.z*10.);
        d=sdSphere(p,.8+cells*.2);
    }
    else if(shape_selector<17.){
        float spiral=atan(p.z,p.x)+p.y*2.;
        d=sdTorus(p,vec2(1.+sin(spiral)*.2,.3));
    }
    else if(shape_selector<18.){
        vec3 q=abs(p);
        d=min(min(
                sdBox(q-vec3(1,0,0),vec3(.2)),
                sdBox(q-vec3(0,1,0),vec3(.2))),
                sdBox(q-vec3(0,0,1),vec3(.2)));
            }
            else if(shape_selector<19.){
                float gyroid=dot(sin(p),cos(p.zxy));
                d=abs(gyroid)-.3;
            }
            else if(shape_selector<20.){
                float menger=sdBox(p,vec3(1.));
                for(int i=0;i<3;i++){
                    vec3 q=mod(p*pow(3.,float(i)),2.)-1.;
                    menger=max(menger,-sdBox(q,vec3(.8)));
                }
                d=menger;
            }
            else if(shape_selector<21.){
                vec3 q=p;
                q.xz=mod(q.xz+1.,2.)-1.;
                d=sdCylinder(q,vec2(.3,1.));
            }
            else if(shape_selector<22.){
                d=sdSphere(p,1.)+noise(p*5.)*.2;
            }
            else if(shape_selector<23.){
                float weave=sin(p.x*5.)*sin(p.z*5.)*.2;
                d=sdTorus(p+vec3(0,weave,0),vec2(1.,.2));
            }
            else if(shape_selector<24.){
                vec3 q=p;
                q.y=mod(q.y+1.,2.)-1.;
                d=sdBox(q,vec3(.2,.8,.2));
            }
            else if(shape_selector<25.){
                float helix=length(p.xz)-1.+sin(atan(p.z,p.x)*3.+p.y*5.)*.2;
                d=max(helix,abs(p.y)-1.);
            }
            else if(shape_selector<26.){
                d=sdSphere(p,1.)*(.8+.2*sin(atan(p.z,p.x)*8.));
            }
            else if(shape_selector<27.){
                vec3 q=p;
                float angle=atan(q.z,q.x);
                q.xz=mat2(cos(angle*3.),-sin(angle*3.),
                sin(angle*3.),cos(angle*3.))*q.xz;
                d=sdBox(q,vec3(.5));
            }
            else if(shape_selector<28.){
                float ripples=sin(length(p.xz)*5.-time)*.2;
                d=sdCylinder(p+vec3(0,ripples,0),vec2(.8,.3));
            }
            else if(shape_selector<29.){
                vec3 q=p;
                q.xz*=mat2(cos(q.y),-sin(q.y),sin(q.y),cos(q.y));
                d=sdBox(q,vec3(.5,1.,.1));
            }
            else if(shape_selector<30.){
                float fractal=1.;
                vec3 q=p;
                for(int i=0;i<4;i++){
                    q=abs(q)-vec3(.3,.5,.3);
                    q*=1.5;
                    fractal*=.6;
                }
                d=sdBox(q,vec3(.3))*fractal;
            }
            else if(shape_selector<31.){
                float flowers=sin(atan(p.z,p.x)*5.)*.2;
                d=sdTorus(p+vec3(0,flowers,0),vec2(1.,.2));
            }
            else if(shape_selector<32.){
                vec3 q=p;
                q.y=abs(q.y);
                float angle=atan(q.z,q.x);
                q.xz=mat2(cos(angle*2.),-sin(angle*2.),
                sin(angle*2.),cos(angle*2.))*q.xz;
                d=sdBox(q,vec3(.2,1.,.2));
            }
            else if(shape_selector<33.){
                float lattice=sin(p.x*10.)*sin(p.y*10.)*sin(p.z*10.);
                d=max(sdBox(p,vec3(1.)),lattice);
            }
            else if(shape_selector<34.){
                vec3 q=p;
                q.xz*=mat2(cos(time),-sin(time),sin(time),cos(time));
                d=min(sdCone(q,vec2(1,1),1.),
                sdCone(q*vec3(1,-1,1),vec2(1,1),1.));
            }
            else if(shape_selector<35.){
                float waves=sin(p.x*5.+time)*cos(p.z*5.)*.2;
                d=sdSphere(p+vec3(0,waves,0),.8);
            }
            else if(shape_selector<36.){
                vec3 q=p;
                q.y=mod(q.y+2.,4.)-2.;
                float twist=q.y;
                q.xz*=mat2(cos(twist),-sin(twist),sin(twist),cos(twist));
                d=sdBox(q,vec3(.5,.1,.5));
            }
            else if(shape_selector<37.){
                float cells=sin(p.x*8.)*sin(p.y*8.)*sin(p.z*8.);
                d=max(sdBox(p,vec3(1.)),abs(cells)-.2);
            }
            else if(shape_selector<38.){
                vec3 q=p;
                float angle=atan(q.z,q.x);
                q.xz=mat2(cos(angle*5.),-sin(angle*5.),
                sin(angle*5.),cos(angle*5.))*q.xz;
                d=sdCapsule(q,vec3(0,-1,0),vec3(0,1,0),.2);
            }
            else if(shape_selector<39.){
                float spiral=atan(p.z,p.x)+length(p.xz)*2.;
                d=max(sdCylinder(p,vec2(1.,.2)),
                sin(spiral*3.)*.1);
            }
            else{
                // Shape 40: Complex recursive structure
                vec3 q=p;
                float scale=1.;
                d=sdSphere(q,1.);
                for(int i=0;i<3;i++){
                    q=abs(q)-.5;
                    q*=1.5;
                    scale*=.5;
                    d=min(d,sdBox(q,vec3(.2))*scale);
                }
            }
            
            // Apply hollow effect if needed
            if(v.hollow_factor>0.){
                float inner=d+v.hollow_factor;
                d=max(d,-inner);
            }
            
            // Apply effects
            d+=sin(length(p.xz)*v.ripple_freq)*v.ripple_amp;
            d+=sin(time*v.wobble_speed)*v.pulse_amount;
            
            return d*scale;
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
                    color=vec4(.1,.1,.1,1.);// Background color
                }else{
                    vec3 p=ro+rd*d;
                    vec3 n=getNormal(p);
                    vec3 col=lighting(p,n,rd);
                    
                    // Apply fog
                    float fog=1.-exp(-d*.1);
                    col=mix(col,vec3(.1),fog);
                    
                    // Apply gamma correction
                    col=pow(col,vec3(1./2.2));
                    
                    color=vec4(col,1.);
                }
            }