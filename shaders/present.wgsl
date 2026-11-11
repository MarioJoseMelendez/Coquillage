@group(0) @binding(0) var world: texture_2d<f32>;

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

@group(0) @binding(1) var<uniform> light: Light;
@group(0) @binding(2) var cover: texture_2d<f32>;

struct VsOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VsOut {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );
    var uvs = array<vec2<f32>, 3>(
        vec2<f32>(0.0, 1.0),
        vec2<f32>(2.0, 1.0),
        vec2<f32>(0.0, -1.0),
    );

    var out: VsOut;
    out.clip = vec4<f32>(positions[vertex_index], 0.0, 1.0);
    out.uv = uvs[vertex_index];
    return out;
}

fn luma_of(c: vec3<f32>) -> f32 {
    return dot(c, vec3<f32>(0.299, 0.587, 0.114));
}

fn hash21(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(127.1, 311.7))) * 43758.5453);
}

fn load_px(dim: vec2<i32>, p: vec2<i32>) -> vec3<f32> {
    let q = clamp(p, vec2<i32>(0), dim - vec2<i32>(1));
    return textureLoad(world, q, 0).rgb;
}

fn disc(px: vec2<f32>, c: vec2<f32>, r: f32) -> f32 {
    let d = length(px - round(c));
    return step(d, r);
}

// 3×5 HUD digits. Pixeli: timers stay 3×5, 1 px gap, shadow not outline (outline fills counters).
fn digit_row(d: i32, y: i32) -> i32 {
    if (y < 0 || y > 4) {
        return 0;
    }
    switch d {
        case 0: { return select(7, 5, y != 0 && y != 4); }
        case 1: {
            if (y == 1) { return 6; }
            if (y == 4) { return 7; }
            return 2;
        }
        case 2: {
            if (y == 0 || y == 2 || y == 4) { return 7; }
            if (y == 1) { return 1; }
            return 4;
        }
        case 3: { return select(7, 1, y == 1 || y == 3); }
        case 4: {
            if (y < 2) { return 5; }
            if (y == 2) { return 7; }
            return 1;
        }
        case 5: {
            if (y == 0 || y == 2 || y == 4) { return 7; }
            if (y == 1) { return 4; }
            return 1;
        }
        case 6: {
            if (y == 0 || y == 2 || y == 4) { return 7; }
            if (y == 1) { return 4; }
            return 5;
        }
        case 7: { return select(7, 1, y != 0); }
        case 8: { return select(7, 5, y == 1 || y == 3); }
        case 9: {
            if (y == 0 || y == 2) { return 7; }
            if (y == 4) { return 7; }
            if (y == 1) { return 5; }
            return 1;
        }
        default: { return 0; }
    }
}

fn digit_ink(d: i32, lx: i32, ly: i32) -> bool {
    if (lx < 0 || lx > 2 || ly < 0 || ly > 4) {
        return false;
    }
    return ((digit_row(d, ly) >> (2 - lx)) & 1) == 1;
}

fn clock_ink(px: vec2<i32>, origin: vec2<i32>, h: i32, m: i32) -> bool {
    let p = px - origin;
    if (p.y < 0 || p.y > 4) {
        return false;
    }
    let digits = array<i32, 4>(h / 10, h - (h / 10) * 10, m / 10, m - (m / 10) * 10);
    let xs = array<i32, 4>(0, 4, 10, 14);
    if (p.x == 8 && (p.y == 1 || p.y == 3)) {
        return true;
    }
    for (var i = 0; i < 4; i++) {
        if (digit_ink(digits[i], p.x - xs[i], p.y)) {
            return true;
        }
    }
    return false;
}

fn sky_open(worldp: vec2<f32>) -> bool {
    let cx = i32(floor(worldp.x / 16.0));
    let cy = i32(floor(worldp.y / 16.0));
    let dim = vec2<i32>(textureDimensions(cover));
    if (cx < 0 || cy < 0 || cx >= 128 || cy >= dim.y) {
        return false;
    }
    return textureLoad(cover, vec2<i32>(cx, cy), 0).r < 0.5;
}

// Falling streak: Y-down world, so subtract time. Bright bead at the BOTTOM,
// tapering tail above — otherwise fract() reads as rain climbing.
fn rain_streak(
    worldp: vec2<f32>,
    time: f32,
    speed: f32,
    period: f32,
    len_px: f32,
    thresh: f32,
    salt: f32,
) -> f32 {
    let col = floor(worldp.x);
    let seed = hash21(vec2<f32>(col, salt));
    let dens = step(thresh, seed);
    let y = worldp.y - time * speed - seed * 53.0;
    let along = fract(y / period);
    let len = len_px / period;
    let in_drop = step(1.0 - len, along);
    let u = saturate((along - (1.0 - len)) / max(len, 0.001));
    let body = u * u;
    let bead = step(1.0 - 1.15 / period, along);
    return dens * in_drop * max(body, bead);
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let dimf = vec2<f32>(textureDimensions(world));
    let dim = vec2<i32>(textureDimensions(world));
    let px = vec2<i32>(min(in.uv * dimf, dimf - vec2<f32>(1.0)));
    let src = textureLoad(world, px, 0);
    let luma = luma_of(src.rgb);
    let hour = light.day_hour;
    let night = hour < 6.0 || hour >= 18.0;
    let shade = light.shadow.xyz;
    let sun = light.sun_tint.xyz;
    let fog = light.fog.xyz;
    let haze = light.shadow.w;

    let warmth = mix(shade, sun, smoothstep(0.18, 0.82, luma));
    let grade_amt = mix(0.16, 0.28, select(1.0, 0.0, !night));
    var rgb = mix(src.rgb, src.rgb * warmth, grade_amt);
    let gray = vec3<f32>(luma_of(rgb));
    rgb = mix(gray, rgb, light.sat);

    let reach = 2.0 + (1.0 - light.elevation) * 5.0;
    let sun_step = vec2<i32>(
        i32(round(-light.sun_dir.x * reach)),
        i32(round(-light.sun_dir.y * reach)),
    );
    let occluder = luma_of(load_px(dim, px + sun_step));
    let blocked = smoothstep(0.05, 0.20, luma - occluder);
    let occl_k = mix(0.16, 0.38, 1.0 - light.elevation) * light.rim.w;
    rgb *= mix(vec3<f32>(1.0), shade, blocked * occl_k);

    let worldp = light.cam_xy + vec2<f32>(f32(px.x), f32(px.y));
    let axis = worldp.x * light.sun_dir.y - worldp.y * light.sun_dir.x;
    let band = step(0.992, fract(axis * 0.018));
    let chop = step(0.90, hash21(floor(worldp / 11.0)));
    let shaft = band * chop * smoothstep(0.78, 0.94, luma) * select(0.004, 0.008, !night);
    rgb += light.disc.xyz * shaft;

    rgb = mix(rgb, fog, haze * (1.0 - luma * 0.45));
    rgb = mix(rgb, vec3<f32>(luma_of(rgb)), light.rain * 0.07);
    rgb = mix(rgb, rgb * vec3<f32>(1.05, 1.05, 1.06), light.rain * 0.08);

    let pxf = vec2<f32>(f32(px.x), f32(px.y));
    let path = select((hour + 6.0) % 12.0 / 12.0, (hour - 6.0) / 12.0, !night);
    let cx = 80.0 + path * 480.0;
    let cy = 26.0 + (1.0 - light.elevation) * 36.0;
    let body = disc(pxf, vec2<f32>(cx, cy), select(5.0, 6.0, !night));
    let halo = disc(pxf, vec2<f32>(cx, cy), select(6.0, 7.0, !night)) - body;
    rgb = mix(rgb, light.disc.xyz, body * 0.82);
    rgb = mix(rgb, light.rim.xyz, halo * 0.45);

    let rain = light.rain;
    if (rain > 0.02) {
        let pale = vec3<f32>(0.86, 0.90, 0.92);
        let far = rain_streak(worldp, light.time, 92.0, 40.0, 3.0, mix(1.0, 0.78, rain), 2.0);
        let mid = rain_streak(worldp, light.time, 148.0, 28.0, 4.5, mix(1.0, 0.86, rain), 5.0);
        let near = rain_streak(worldp, light.time, 210.0, 22.0, 6.0, mix(1.0, 0.93, rain), 9.0);
        let drops = saturate(far * 0.40 + mid * 0.70 + near * 1.0);
        rgb = mix(rgb, pale, drops * rain * 0.28);

        if (sky_open(worldp)) {
            rgb = mix(rgb, rgb * vec3<f32>(0.82, 0.88, 0.90), rain * 0.10);
            let chroma = length(src.rgb - vec3<f32>(luma));
            let wet = rain * smoothstep(0.08, 0.02, chroma) * smoothstep(0.38, 0.58, luma);
            rgb = mix(rgb, rgb * vec3<f32>(0.90, 0.96, 1.03) + vec3<f32>(0.02, 0.03, 0.04), wet * 0.35);

            let cell = floor(worldp / 16.0);
            let hs = hash21(cell + vec2<f32>(5.0, 11.0));
            if (hs > 0.86) {
                let t0 = fract(light.time * (1.35 + hs * 0.55) + hs);
                if (t0 < 0.18) {
                    let site = cell * 16.0 + vec2<f32>(3.0 + hs * 10.0, 3.0 + hash21(cell + 17.0) * 10.0);
                    let d = length(worldp - site);
                    let rad = t0 * 3.2;
                    let ring = saturate(1.0 - abs(d - rad) * 2.4) * (1.0 - t0 * 4.0);
                    let core = step(d, 1.05) * saturate(1.0 - t0 * 6.0);
                    rgb = mix(rgb, pale, max(ring, core) * rain * 0.32);
                }
            }
        }
    }

    let uvn = in.uv * 2.0 - 1.0;
    let canopy = 1.0 - dot(uvn, uvn) * 0.05;
    let rim = max(abs(uvn.x), abs(uvn.y));
    let day_k = select(
        0.0,
        sin(clamp((hour - 6.0) / 12.0, 0.0, 1.0) * 3.14159265),
        hour > 6.0 && hour < 18.0
    );
    let aura = smoothstep(0.78, 1.0, rim) * mix(0.18, 0.05, day_k);
    rgb = mix(rgb, mix(fog, vec3<f32>(0.72, 0.76, 0.78), 0.40), aura);
    rgb *= canopy;

    // Clock rides with the sun, a few pixels below. Fill from the disc, 1 px purple shadow.
    let ih = i32(floor(hour)) % 24;
    let im = i32(floor(fract(hour) * 60.0)) % 60;
    let clock_o = vec2<i32>(i32(round(cx)) - 8, i32(round(cy)) + 9);
    let pxi = vec2<i32>(px.x, px.y);
    let ink = clock_ink(pxi, clock_o, ih, im);
    let sh = clock_ink(pxi, clock_o + vec2<i32>(1, 1), ih, im) && !ink;
    let foam = vec3<f32>(214.0, 224.0, 210.0) / 255.0;
    let leaf_sh = vec3<f32>(56.0, 40.0, 72.0) / 255.0;
    if (sh) {
        rgb = mix(rgb, leaf_sh, 0.55);
    }
    if (ink) {
        rgb = mix(rgb, mix(light.disc.xyz, foam, 0.45), 0.78);
    }
    return vec4<f32>(rgb, src.a);
}
