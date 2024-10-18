@group(0) @binding(1) var r_textures: binding_array<texture_2d<f32>>;
@group(0) @binding(2) var r_sampler: sampler;

@fragment
fn main(
    @builtin(position) in_position: vec4<f32>,
    @location(0) coord: vec2<f32>,
    @location(1) index: u32,
) -> @location(0) vec4<f32> {
    return textureSample(r_textures[index], r_sampler, coord);
}