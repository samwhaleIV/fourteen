struct CameraUniform {
    view_projection: mat4x4<f32>,
};

@group(1) @binding(0) var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec2<f32>,
};

struct InstanceInput {
    @location(1) offset: vec2<f32>,
    @location(2) scale: vec2<f32>,
    @location(3) uv_offset: vec2<f32>,
    @location(4) uv_scale: vec2<f32>,
    @location(5) color: vec4<f32>,
    @location(6) rotation: f32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
};

fn rotate(rotation: f32) -> mat2x2<f32> {
    let c = cos(rotation);
    let s = sin(rotation);
    return mat2x2<f32>(vec2<f32>(c,-s),vec2<f32>(s,c));
}

@vertex fn vs_main(vertex: VertexInput,instance: InstanceInput) -> VertexOutput {
    var out: VertexOutput;

    let world_position = rotate(instance.rotation) * vertex.position * instance.scale + instance.offset;
    out.clip_position = camera.view_projection * vec4<f32>(world_position,0.0,1.0);

    out.uv = (vertex.position + vec2<f32>(0.5)) * instance.uv_scale + instance.uv_offset;
    out.color = instance.color;
    return out;
}

@group(0) @binding(0) var t_diffuse: texture_2d<f32>;
@group(0) @binding(1) var s_diffuse: sampler;

fn linear_to_srgb(linear: vec4<f32>) -> vec4<f32> {
    let color_linear = linear.rgb;
    let selector = ceil(color_linear - vec3<f32>(0.0031308));
    let under = 12.92 * color_linear;
    let over = 1.055 * pow(color_linear,vec3<f32>(1.0/2.4)) - 0.055;
    let result = mix(under,over,selector);
    return vec4<f32>(result,linear.a);
}

@fragment fn fs_no_srgb(fragment: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(t_diffuse,s_diffuse,fragment.uv) * fragment.color;
    return color;
}

@fragment fn fs_to_srgb(fragment: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(t_diffuse,s_diffuse,fragment.uv) * fragment.color;
    return linear_to_srgb(color);
}
