struct Params {
    bounds: vec2f,
    min_coord: vec2f,
}

@group(0) @binding(1) var r_textures: binding_array<texture_2d<f32>>;
@group(0) @binding(2) var r_sampler: sampler;
@group(0) @binding(3) var<storage, read> r_params: array<Params>;

@fragment
fn main(
    @builtin(position) in_position: vec4f,
    @location(0) coord: vec2f,
    @location(1) index: u32,
) -> @location(0) vec4f {
    var param = r_params[index];
    var norm_coord = (coord - fma(param.min_coord, vec2f(0.5, -0.5), vec2f(0.5, 0.5))) / param.bounds;
    return textureSample(r_textures[index], r_sampler, norm_coord);
}