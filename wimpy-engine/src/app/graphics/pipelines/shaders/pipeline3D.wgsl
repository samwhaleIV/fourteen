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
    @location(3) transform_0: vec4<f32>,
    @location(4) transform_1: vec4<f32>,
    @location(5) transform_2: vec4<f32>,
    @location(6) transform_3: vec4<f32>,
    @location(7) diffuse_color: vec4<f32>,
    @location(8) lightmap_color: vec4<f32>
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

    var transform = mat4x4<f32>(
        instance.transform_0,
        instance.transform_1,
        instance.transform_2,
        instance.transform_3
    );

    out.clip_position = camera.view_projection * transform * vec4<f32>(vertex.position,1.0);

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

fn linear_to_srgb(linear: vec4<f32>) -> vec4<f32> {
    let color_linear = linear.rgb;
    let selector = ceil(color_linear - vec3<f32>(0.0031308));
    let under = 12.92 * color_linear;
    let over = 1.055 * pow(color_linear,vec3<f32>(1.0/2.4)) - 0.055;
    let result = mix(under,over,selector);
    return vec4<f32>(result,linear.a);
}

fn get_fragment_color(fragment: VertexOutput) -> vec4<f32> {
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

@fragment fn fs_no_srgb(fragment: VertexOutput) -> @location(0) vec4<f32> {
    let color = get_fragment_color(fragment);
    return color;
}

@fragment fn fs_to_srgb(fragment: VertexOutput) -> @location(0) vec4<f32> {
    let color = get_fragment_color(fragment);
    return linear_to_srgb(color);
}
