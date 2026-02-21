struct CameraUniform {
    view_projection: mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex fn vs_main(vertex: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera.view_projection * vec4<f32>(vertex.position,0.0,1.0);
    out.color = vertex.color;
    return out;
}

fn linear_to_srgb(linear: vec4<f32>) -> vec4<f32> {
    let color_linear = linear.rgb;
    let selector = ceil(color_linear - vec3<f32>(0.0031308));
    let under = 12.92 * color_linear;
    let over = 1.055 * pow(color_linear,vec3<f32>(1.0/2.4)) - 0.055;
    let result = mix(under,over,selector);
    return vec4<f32>(result,linear.a);
}

@fragment fn fs_no_srgb(fragment: VertexOutput) -> @location(0) vec4<f32> {
    return fragment.color;
}

@fragment fn fs_to_srgb(fragment: VertexOutput) -> @location(0) vec4<f32> {
    return linear_to_srgb(fragment.color);
}
