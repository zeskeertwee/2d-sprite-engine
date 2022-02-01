struct CameraUniform {
    proj: mat4x4<f32>;
};

[[group(0), binding(0)]]
var<uniform> camera: CameraUniform;

struct PushConstants {
    model: mat4x4<f32>;
};

var<push_constant> push_constant: PushConstants;

struct VertexInput {
    [[location(0)]] position: vec2<f32>;
    [[location(1)]] tex_coords: vec2<f32>;
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] tex_coords: vec2<f32>;
};

[[stage(vertex)]]
fn vs_main(
    model: VertexInput
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.clip_position = camera.proj * push_constant.model * vec4<f32>(model.position.xy, 0.0, 1.0);
    // if we don't do this, it ends up outside of the 0-1 range that wgpu requires for something to be drawn
    //out.clip_position.z = abs(model.position.z) / 10000.0;
    out.clip_position.z = 0.0;
    return out;
}

[[group(1), binding(0)]]
var t_texture: texture_2d<f32>;
[[group(1), binding(1)]]
var t_sampler: sampler;

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    return textureSample(t_texture, t_sampler, in.tex_coords);
}
