struct AlignedVec2 {
    data: vec2f,
    padding: vec2f,
}

struct Params {
    bounds: array<AlignedVec2, 16>,
    min_coords: array<AlignedVec2, 16>,
}

@group(0) @binding(1) var r_textures: binding_array<texture_2d<f32>>;
@group(0) @binding(2) var r_sampler: sampler;
@group(0) @binding(3) var<uniform> r_params: Params;

@fragment
fn main(
    @builtin(position) in_position: vec4f,
    @location(0) coord: vec2f,
    @location(1) index: u32,
) -> @location(0) vec4f {
    var norm_coord = (coord - fma(r_params.min_coords[index].data, vec2f(0.5, -0.5), vec2f(0.5, 0.5))) / r_params.bounds[index].data;
    return textureSample(r_textures[index], r_sampler, norm_coord);
}