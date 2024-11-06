struct VertexOutput {
    @location(0) coord: vec2f,
    @location(1) index: u32,
    @builtin(position) position: vec4f,
}

struct Params {
    transform: mat4x4<f32>,
}

@group(0) @binding(0) var<uniform> r_params: Params;

@vertex
fn main(
    @builtin(vertex_index) in_vertex_index: u32,
    @location(0) coord: vec2f
) -> VertexOutput {
    var out: VertexOutput;
    out.coord = fma(coord, vec2f(0.5, -0.5), vec2f(0.5, 0.5));
    out.index = (in_vertex_index + 2u) / 6u;
    out.position = r_params.transform * vec4f(coord, 0.0, 1.0);
    return out;
}