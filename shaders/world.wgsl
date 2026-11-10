struct Camera {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> camera: Camera;
@group(0) @binding(1) var atlas: texture_2d<f32>;

struct VsIn {
    @location(0) pos: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) world_xy: vec2<f32>,
    @location(3) atlas_xy: vec2<f32>,
};

struct VsOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) atlas_xy: vec2<f32>,
};

const TILE: f32 = 32.0;

@vertex
fn vs_main(in: VsIn) -> VsOut {
    var out: VsOut;
    let world = vec2<f32>(
        in.world_xy.x + in.pos.x * TILE,
        in.world_xy.y + in.pos.y * TILE,
    );
    out.clip = camera.view_proj * vec4<f32>(world, 0.0, 1.0);
    out.uv = in.uv;
    out.atlas_xy = in.atlas_xy;
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let local = vec2<i32>(clamp(in.uv * TILE, vec2<f32>(0.0), vec2<f32>(TILE - 1.0)));
    let origin = vec2<i32>(in.atlas_xy);
    return textureLoad(atlas, origin + local, 0);
}
