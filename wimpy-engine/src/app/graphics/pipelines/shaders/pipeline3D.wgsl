struct CameraUniform {
    view_projection: mat4x4<f32>,
};

@group(0) @binding(0) var t_diffuse: texture_2d<f32>;
@group(0) @binding(1) var s_diffuse: sampler;
@group(0) @binding(2) var t_lightmap: texture_2d<f32>;
@group(0) @binding(3) var s_lightmap: sampler;

@group(1) @binding(0) var<uniform> camera: CameraUniform;

@group(2) @binding(0) var<storage,read> vertices: array<VertexInput>;
@group(2) @binding(1) var<storage,read> indices: array<u32>;
@group(2) @binding(2) var<storage,read> instances: array<InstanceInput>;

struct VertexInput {
    uv_diffuse: vec2<f32>,
    uv_lightmap: vec2<f32>,
    position: vec3<f32>,
};

struct InstanceInput {
    // x, y, width, height
    uv_diffuse: vec4<f32>,

    // x, y, width, height
    uv_lightmap: vec4<f32>,

    transform_0: vec4<f32>,
    transform_1: vec4<f32>,
    transform_2: vec4<f32>,
    transform_3: vec4<f32>,

    base_vertex: u32,
    index_start: u32,
    index_count: u32
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv_diffuse: vec2<f32>,
    @location(1) uv_lightmap: vec2<f32>,
};

fn uv_within(vertex_uv: vec2<f32>,uv: vec4<f32>) -> vec2<f32> {
    return fma(uv.zw,vertex_uv,uv.xy);
}

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_id: u32,
    @builtin(instance_index) instance_id: u32
) -> VertexOutput {
    var out: VertexOutput;

    let instance: InstanceInput = instances[instance_id];

    let index_id = instance.index_start + vertex_id;

    if vertex_id >= instance.index_count {
        out.clip_position = vec4<f32>(0.0);
        out.uv_diffuse = vec2<f32>(0.0);
        out.uv_lightmap = vec2<f32>(0.0);
        return out;
    }

    let vertex: VertexInput = vertices[indices[index_id] + instance.base_vertex];

    let transform = mat4x4<f32>(
        instance.transform_0,
        instance.transform_1,
        instance.transform_2,
        instance.transform_3
    );

    out.clip_position = camera.view_projection * transform * vec4<f32>(vertex.position, 1.0);

    out.uv_diffuse = uv_within(vertex.uv_diffuse,instance.uv_diffuse);
    out.uv_lightmap = uv_within(vertex.uv_lightmap,instance.uv_lightmap);

    return out;
}

fn get_fragment_color(fragment: VertexOutput) -> vec4<f32> {
    var diffuse_sample = textureSample(
        t_diffuse,
        s_diffuse,
        fragment.uv_diffuse
    );

    var lightmap_sampler = textureSample(
        t_lightmap,
        s_lightmap,
        fragment.uv_lightmap
    );

    return diffuse_sample * lightmap_sampler;
}

@fragment
fn fs_main(fragment: VertexOutput) -> @location(0) vec4<f32> {
    let color = get_fragment_color(fragment);
    return color;
}
