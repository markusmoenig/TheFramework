struct VertexOutput {
    @location(0) coord: vec2<f32>,
    @location(1) index: u32,
    @builtin(position) position: vec4<f32>,
}

struct Transform {
    transform: mat4x4<f32>,
}

@group(0) @binding(0) var<uniform> r_transform: Transform;

@vertex
fn main(
    @location(0) position: vec2<f32>,
    @builtin(vertex_index) in_vertex_index: u32
) -> VertexOutput {
    var out: VertexOutput;
    out.coord = fma(position, vec2<f32>(0.5, -0.5), vec2<f32>(0.5, 0.5));
    out.index = (in_vertex_index + 2u) / 6u;
    out.position = r_transform.transform * vec4<f32>(position, 0.0, 1.0);
    return out;
}