struct Uniforms {
    view_proj: mat4x4<f32>,
    point_size: f32,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct PointInput {
    @location(0) position: vec3<f32>,
    @location(1) color: u32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) point_color: vec4<f32>,
};

@vertex
fn vs_main(input: PointInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = uniforms.view_proj * vec4<f32>(input.position, 1.0);

    // Unpack RGBA8 color
    let r = f32(input.color & 0xFFu) / 255.0;
    let g = f32((input.color >> 8u) & 0xFFu) / 255.0;
    let b = f32((input.color >> 16u) & 0xFFu) / 255.0;
    let a = f32((input.color >> 24u) & 0xFFu) / 255.0;
    out.point_color = vec4<f32>(r, g, b, a);

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.point_color;
}
