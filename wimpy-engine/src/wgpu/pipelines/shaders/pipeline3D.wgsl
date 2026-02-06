struct CameraUniform {
    view_projection: mat4x4<f32>,
};
@group(1) @binding(0) var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) diffuse_uv: vec2<f32>,
    @location(1) lightmap_uv: vec2<f32>,
    @location(2) position: vec3<f32>,
};

struct InstanceInput {
    @location(3) diffuse_color: vec4<f32>,
    @location(4) lightmap_color: vec4<f32>
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) diffuse_uv: vec2<f32>,
    @location(1) lightmap_uv: vec2<f32>,
    @location(2) diffuse_color: vec4<f32>,
    @location(3) lightmap_color: vec4<f32>
};

@vertex fn vs_main(vertex: VertexInput,instance: InstanceInput) -> VertexOutput {
    var out: VertexOutput;

    out.clip_position = camera.view_projection * vec4<f32>(vertex.position,1.0);

    out.diffuse_uv = vertex.diffuse_uv;
    out.lightmap_uv = vertex.lightmap_uv;

    out.diffuse_color = instance.diffuse_color;
    out.lightmap_color = instance.lightmap_color;

    return out;
}

@group(0) @binding(0) var t_diffuse: texture_2d<f32>;
@group(0) @binding(1) var s_diffuse: sampler;
@group(0) @binding(2) var t_lightmap: texture_2d<f32>;
@group(0) @binding(3) var s_lightmap: sampler;

@fragment fn fs_main(fragment: VertexOutput) -> @location(0) vec4<f32> {
    var diffuse_sample = textureSample(
        t_diffuse,
        s_diffuse,
        fragment.diffuse_uv
    ) * fragment.diffuse_color;

    var lightmap_sampler = textureSample(
        t_lightmap,
        s_lightmap,
        fragment.lightmap_uv
    ) * fragment.lightmap_color;

    return diffuse_sample * lightmap_sampler;
}
