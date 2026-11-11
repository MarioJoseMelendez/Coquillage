use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde::Deserialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Terrain,
    Decal,
    Prop,
}

#[derive(Clone, Debug)]
pub struct Sprite {
    pub id: String,
    pub role: Role,
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

#[derive(Clone, Copy, Debug)]
pub struct Frame {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

pub struct Atlas {
    pub pixels: Vec<u8>,
    pub normals: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub frames: HashMap<String, Frame>,
}

#[derive(Deserialize)]
struct Manifest {
    sprites: Vec<ManifestSprite>,
}

#[derive(Deserialize)]
struct ManifestSprite {
    id: String,
    file: String,
    role: String,
}

pub fn terrain_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/terrain")
}

pub fn props_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/props")
}

pub fn load_catalog() -> Result<Vec<Sprite>> {
    let mut sprites = load_catalog_from(&terrain_dir())?;
    if props_dir().join("manifest.json").is_file() {
        sprites.extend(load_catalog_from(&props_dir())?);
    }
    Ok(sprites)
}

pub fn load_catalog_from(dir: &Path) -> Result<Vec<Sprite>> {
    let manifest_path = dir.join("manifest.json");
    let text = std::fs::read_to_string(&manifest_path)
        .with_context(|| format!("read {}", manifest_path.display()))?;
    let manifest: Manifest = serde_json::from_str(&text)
        .with_context(|| format!("parse {}", manifest_path.display()))?;

    let mut sprites = Vec::with_capacity(manifest.sprites.len());
    for entry in manifest.sprites {
        let role = match entry.role.as_str() {
            "terrain" => Role::Terrain,
            "decal" => Role::Decal,
            "prop" => Role::Prop,
            other => bail!("unknown sprite role `{other}` in {}", entry.id),
        };
        let path = dir.join(&entry.file);
        let image = image::open(&path)
            .with_context(|| format!("load PNG {}", path.display()))?
            .into_rgba8();
        let (width, height) = image.dimensions();
        if width == 0 || height == 0 {
            bail!("{} has empty size", entry.id);
        }
        sprites.push(Sprite {
            id: entry.id,
            role,
            width,
            height,
            pixels: image.into_raw(),
        });
    }
    Ok(sprites)
}

pub fn pack_atlas(sheets: &[(String, u32, u32, Vec<u8>)]) -> Result<Atlas> {
    if sheets.is_empty() {
        bail!("cannot pack an empty atlas");
    }

    let mut order: Vec<usize> = (0..sheets.len()).collect();
    order.sort_by_key(|&i| std::cmp::Reverse(sheets[i].2));

    let atlas_width = next_pow2(
        sheets
            .iter()
            .map(|(_, w, _, _)| *w)
            .max()
            .unwrap_or(32)
            .max(512),
    );
    let mut x = 0u32;
    let mut y = 0u32;
    let mut row_h = 0u32;
    let mut placed = vec![(0u32, 0u32); sheets.len()];
    const PAD: u32 = 1;

    for i in order {
        let w = sheets[i].1;
        let h = sheets[i].2;
        if x > 0 && x + w + PAD > atlas_width {
            x = 0;
            y += row_h;
            row_h = 0;
        }
        placed[i] = (x, y);
        x += w + PAD;
        row_h = row_h.max(h + PAD);
    }
    let atlas_height = next_pow2(y + row_h);

    let mut pixels = vec![0u8; (atlas_width * atlas_height * 4) as usize];
    let mut frames = HashMap::new();
    for (i, (id, w, h, src)) in sheets.iter().enumerate() {
        let (ox, oy) = placed[i];
        blit(&mut pixels, atlas_width, ox, oy, src, *w, *h);
        frames.insert(
            id.clone(),
            Frame {
                x: ox,
                y: oy,
                w: *w,
                h: *h,
            },
        );
    }
    for frame in frames.values() {
        extrude_frame(&mut pixels, atlas_width, atlas_height, frame);
    }

    let normals = bake_normals(&pixels, atlas_width, atlas_height, &frames);

    Ok(Atlas {
        pixels,
        normals,
        width: atlas_width,
        height: atlas_height,
        frames,
    })
}

fn bake_normals(
    albedo: &[u8],
    atlas_w: u32,
    atlas_h: u32,
    frames: &HashMap<String, Frame>,
) -> Vec<u8> {
    let mut out = vec![0u8; (atlas_w * atlas_h * 4) as usize];
    for i in 0..(atlas_w * atlas_h) as usize {
        let o = i * 4;
        out[o] = 128;
        out[o + 1] = 128;
        out[o + 2] = 255;
    }
    for (id, frame) in frames {
        let strength = if id.starts_with("shadow") {
            0.0
        } else if id.starts_with("t:")
            || id.starts_with("p:")
            || id.starts_with("w:")
            || id == "pebble"
            || id == "tuft"
        {
            0.22
        } else {
            0.58
        };
        bake_frame_normals(albedo, &mut out, atlas_w, frame, strength);
    }
    let _ = atlas_h;
    out
}

fn bake_frame_normals(
    albedo: &[u8],
    out: &mut [u8],
    atlas_w: u32,
    frame: &Frame,
    strength: f32,
) {
    for ly in 0..frame.h {
        for lx in 0..frame.w {
            let x = frame.x + lx;
            let y = frame.y + ly;
            let a = pixel_alpha(albedo, atlas_w, x, y);
            let dst = ((y * atlas_w + x) * 4) as usize;
            if a < 20 {
                out[dst + 3] = a;
                continue;
            }
            if strength <= 0.001 {
                out[dst] = 128;
                out[dst + 1] = 128;
                out[dst + 2] = 255;
                out[dst + 3] = a;
                continue;
            }
            let luma = |ox: i32, oy: i32| -> f32 {
                let px = (x as i32 + ox).clamp(frame.x as i32, (frame.x + frame.w - 1) as i32) as u32;
                let py = (y as i32 + oy).clamp(frame.y as i32, (frame.y + frame.h - 1) as i32) as u32;
                pixel_luma_or(albedo, atlas_w, px, py, pixel_luma_or(albedo, atlas_w, x, y, 0.5))
            };
            let dx = (luma(1, -1) + 2.0 * luma(1, 0) + luma(1, 1))
                - (luma(-1, -1) + 2.0 * luma(-1, 0) + luma(-1, 1));
            let dy = (luma(-1, 1) + 2.0 * luma(0, 1) + luma(1, 1))
                - (luma(-1, -1) + 2.0 * luma(0, -1) + luma(1, -1));
            let nx = -dx * strength;
            let ny = -dy * strength;
            let nz = 1.0;
            let len = (nx * nx + ny * ny + nz * nz).sqrt().max(0.0001);
            out[dst] = (((nx / len) * 0.5 + 0.5).clamp(0.0, 1.0) * 255.0) as u8;
            out[dst + 1] = (((ny / len) * 0.5 + 0.5).clamp(0.0, 1.0) * 255.0) as u8;
            out[dst + 2] = (((nz / len) * 0.5 + 0.5).clamp(0.0, 1.0) * 255.0) as u8;
            out[dst + 3] = a;
        }
    }
}

fn pixel_alpha(pixels: &[u8], atlas_w: u32, x: u32, y: u32) -> u8 {
    pixels[((y * atlas_w + x) * 4 + 3) as usize]
}

fn pixel_luma_or(pixels: &[u8], atlas_w: u32, x: u32, y: u32, fallback: f32) -> f32 {
    let i = ((y * atlas_w + x) * 4) as usize;
    let a = pixels[i + 3];
    if a < 20 {
        return fallback;
    }
    let r = pixels[i] as f32 / 255.0;
    let g = pixels[i + 1] as f32 / 255.0;
    let b = pixels[i + 2] as f32 / 255.0;
    0.299 * r + 0.587 * g + 0.114 * b
}

fn extrude_frame(pixels: &mut [u8], atlas_w: u32, atlas_h: u32, frame: &Frame) {
    let copy = |pixels: &mut [u8], sx: u32, sy: u32, dx: u32, dy: u32| {
        if dx >= atlas_w || dy >= atlas_h {
            return;
        }
        let si = ((sy * atlas_w + sx) * 4) as usize;
        let di = ((dy * atlas_w + dx) * 4) as usize;
        pixels.copy_within(si..si + 4, di);
    };
    if frame.x > 0 {
        for ly in 0..frame.h {
            copy(pixels, frame.x, frame.y + ly, frame.x - 1, frame.y + ly);
        }
    }
    let right = frame.x + frame.w;
    if right < atlas_w {
        for ly in 0..frame.h {
            copy(
                pixels,
                right.saturating_sub(1),
                frame.y + ly,
                right,
                frame.y + ly,
            );
        }
    }
    if frame.y > 0 {
        for lx in 0..frame.w {
            copy(pixels, frame.x + lx, frame.y, frame.x + lx, frame.y - 1);
        }
    }
    let bottom = frame.y + frame.h;
    if bottom < atlas_h {
        for lx in 0..frame.w {
            copy(
                pixels,
                frame.x + lx,
                bottom.saturating_sub(1),
                frame.x + lx,
                bottom,
            );
        }
    }
}

fn blit(dst: &mut [u8], dst_w: u32, ox: u32, oy: u32, src: &[u8], w: u32, h: u32) {
    for row in 0..h {
        let src_off = (row * w * 4) as usize;
        let dst_off = (((oy + row) * dst_w + ox) * 4) as usize;
        let bytes = (w * 4) as usize;
        dst[dst_off..dst_off + bytes].copy_from_slice(&src[src_off..src_off + bytes]);
    }
}

fn next_pow2(value: u32) -> u32 {
    value.next_power_of_two().max(1)
}

pub fn sample(sprite: &Sprite, x: u32, y: u32) -> [u8; 4] {
    let x = x.min(sprite.width.saturating_sub(1));
    let y = y.min(sprite.height.saturating_sub(1));
    let i = ((y * sprite.width + x) * 4) as usize;
    sprite.pixels[i..i + 4].try_into().unwrap_or([0, 0, 0, 0])
}
