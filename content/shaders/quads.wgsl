struct CameraUniform {
    view_projection: mat4x4<f32>,
};
@group(1) @binding(0)
var<uniform> camera: CameraUniform;

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

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment fn fs_main(fragment: VertexOutput) -> @location(0) vec4<f32> {
    if (fragment.color.a < 0.005) {
        //discard;
    }
    return textureSample(t_diffuse,s_diffuse,fragment.uv) * fragment.color;
}
