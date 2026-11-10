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

    Ok(Atlas {
        pixels,
        width: atlas_width,
        height: atlas_height,
        frames,
    })
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
