struct Camera {
    view_proj: mat4x4<f32>,
};

struct Light {
    sun_dir: vec2<f32>,
    elevation: f32,
    time: f32,
    cam_xy: vec2<f32>,
    day_hour: f32,
    sat: f32,
    shadow: vec4<f32>,
    sun_tint: vec4<f32>,
    fog: vec4<f32>,
    disc: vec4<f32>,
    rim: vec4<f32>,
    wind: vec2<f32>,
    rain: f32,
    _weather_pad: f32,
};

@group(0) @binding(0) var<uniform> camera: Camera;
@group(0) @binding(1) var atlas: texture_2d<f32>;
@group(0) @binding(2) var normals: texture_2d<f32>;
@group(0) @binding(3) var<uniform> light: Light;

struct VsIn {
    @location(0) pos: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) world_xy: vec2<f32>,
    @location(3) atlas_xy: vec2<f32>,
    @location(4) size_xy: vec2<f32>,
    @location(5) atlas_wh: vec2<f32>,
    @location(6) alpha: f32,
    @location(7) flags: f32,
};

struct VsOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) atlas_xy: vec2<f32>,
    @location(2) atlas_wh: vec2<f32>,
    @location(3) alpha: f32,
    @location(4) flags: f32,
    @location(5) world_xy: vec2<f32>,
};

@vertex
fn vs_main(in: VsIn) -> VsOut {
    var out: VsOut;
    let world = vec2<f32>(
        in.world_xy.x + in.pos.x * in.size_xy.x,
        in.world_xy.y + in.pos.y * in.size_xy.y,
    );
    out.clip = camera.view_proj * vec4<f32>(world, 0.0, 1.0);
    out.uv = in.uv;
    out.atlas_xy = in.atlas_xy;
    out.atlas_wh = in.atlas_wh;
    out.alpha = in.alpha;
    out.flags = in.flags;
    out.world_xy = world;
    return out;
}

fn hash21(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(127.1, 311.7))) * 43758.5453);
}

fn luma_of(c: vec3<f32>) -> f32 {
    return dot(c, vec3<f32>(0.299, 0.587, 0.114));
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    var local0 = vec2<i32>(clamp(in.uv * in.atlas_wh, vec2<f32>(0.0), in.atlas_wh - vec2<f32>(1.0)));
    if (in.flags > 1.55) {
        let flex = 1.0 - in.uv.y;
        let gust = sin(light.time * 1.22 + in.world_xy.x * 0.034 + in.world_xy.y * 0.027);
        let amp = clamp(in.flags - 1.7, 0.2, 2.5);
        let sway = light.wind.x * gust * flex * amp;
        local0.x = clamp(local0.x + i32(round(sway)), 0, i32(in.atlas_wh.x) - 1);
    }
    let origin = vec2<i32>(in.atlas_xy);
    let t = light.time;
    let water = in.flags > 0.5 && in.flags < 1.5;

    let base = textureLoad(atlas, origin + local0, 0);
    if (base.a < 0.04) {
        discard;
    }

    var color = base;
    var local = local0;
    let foam_col = vec3<f32>(0.84, 0.88, 0.82);
    let water_col = vec3<f32>(0.18, 0.44, 0.47);
    if (water) {
        let px = floor(in.world_xy);
        let h = hash21(px);
        let wave = sin(t * 2.35 + h * 6.2832);
        let ox = i32(round(wave * 2.0 + sin(t * 1.1 + h * 19.0) * 0.5));
        let oy = i32(round(sin(t * 1.7 + h * 11.0) * 1.5));
        let shifted_uv = vec2<i32>(
            clamp(local0.x + ox, 0, i32(in.atlas_wh.x) - 1),
            clamp(local0.y + oy, 0, i32(in.atlas_wh.y) - 1),
        );
        let shifted = textureLoad(atlas, origin + shifted_uv, 0);
        if (shifted.a >= 0.04) {
            color = shifted;
            local = shifted_uv;
        }
    }

    if (color.a < 0.72) {
        return vec4<f32>(color.rgb, color.a * in.alpha);
    }

    let n_enc = textureLoad(normals, origin + local, 0).xyz;
    var n = n_enc * 2.0 - 1.0;
    n = normalize(n);
    let L = normalize(vec3<f32>(light.sun_dir.x, light.sun_dir.y, light.elevation * 1.35));
    let wrap = max(dot(n, L), 0.0) * 0.5 + 0.5;
    let k = 0.70 + light.elevation * 0.65;
    let lit = pow(wrap, k);
    var rgb = mix(color.rgb * light.shadow.xyz, color.rgb * light.sun_tint.xyz, lit);

    if (water) {
        let luma = luma_of(color.rgb);
        var n_hi = luma;
        for (var i = 0; i < 4; i++) {
            let d = vec2<i32>(select(-1, 1, i < 2), select(-1, 1, i % 2 == 0));
            let q = vec2<i32>(
                clamp(local0.x + d.x * 2, 0, i32(in.atlas_wh.x) - 1),
                clamp(local0.y + d.y * 2, 0, i32(in.atlas_wh.y) - 1),
            );
            let s = textureLoad(atlas, origin + q, 0);
            if (s.a >= 0.04) {
                n_hi = max(n_hi, luma_of(s.rgb));
            }
        }
        let wave = sin(t * 2.35 + hash21(floor(in.world_xy)) * 6.2832);
        let crawl = hash21(floor(in.world_xy + vec2<f32>(t * 6.0, t * 2.2)));
        let is_foam = luma > 0.62;
        let near_foam = n_hi > 0.62;
        if (is_foam && wave < 0.12) {
            rgb = mix(rgb, water_col, 0.42 + crawl * 0.35);
        } else if (is_foam) {
            rgb = mix(rgb, foam_col, 0.22 + crawl * 0.20);
        } else if (near_foam && wave > 0.18 && crawl > 0.32) {
            rgb = mix(rgb, foam_col, 0.40 + wave * 0.28);
        }

        let dir = light.sun_dir;
        let spark = step(0.993, hash21(floor(in.world_xy)))
            + step(0.993, hash21(floor(in.world_xy - dir * 1.5)));
        rgb = mix(rgb, light.disc.xyz, min(spark, 1.0) * (0.18 + light.elevation * 0.12));
    }

    return vec4<f32>(rgb, color.a * in.alpha);
}
