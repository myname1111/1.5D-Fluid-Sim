@group(0) @binding(0)
var height_map_texture: texture_1d<f32>;
@group(0) @binding(1)
var height_map_sampler: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.color = model.color;
    out.clip_position = vec4<f32>(model.position, 1.0);
    return out;
}

// Fragment shader

const MAX_HEIGHT: f32 = 5.0;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var tex_coords: f32 = in.color.x;
    var height: f32 = textureSample(height_map_texture, height_map_sampler, tex_coords).x / MAX_HEIGHT;
    if height > in.color.y {
        return vec4<f32>(0.0, 0.0, 1.0, 1.0);
    } else {
        return vec4<f32>(1.0, 1.0, 1.0, 1.0);
    }
}
