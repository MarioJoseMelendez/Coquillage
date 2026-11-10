use std::collections::HashMap;

use anyhow::{bail, Context, Result};
use bytemuck::{Pod, Zeroable};

use crate::assets::{self, Atlas, Frame, Role, Sprite};

pub const TILE_SIZE: u32 = 32;
pub const LOGICAL_WIDTH: f32 = 640.0;
pub const LOGICAL_HEIGHT: f32 = 360.0;
pub const MAP_WIDTH: u32 = 64;
pub const MAP_HEIGHT: u32 = 40;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Vertex {
    pub pos: [f32; 2],
    pub uv: [f32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Instance {
    pub world_xy: [f32; 2],
    pub atlas_xy: [f32; 2],
    pub size_xy: [f32; 2],
    pub alpha: f32,
    pub _pad: f32,
}

pub struct Terrain {
    pub atlas: Atlas,
    pub instances: Vec<Instance>,
    pub occluders: Vec<u32>,
}

pub fn quad_vertices() -> [Vertex; 6] {
    [
        Vertex {
            pos: [0.0, 0.0],
            uv: [0.0, 0.0],
        },
        Vertex {
            pos: [1.0, 0.0],
            uv: [1.0, 0.0],
        },
        Vertex {
            pos: [1.0, 1.0],
            uv: [1.0, 1.0],
        },
        Vertex {
            pos: [0.0, 0.0],
            uv: [0.0, 0.0],
        },
        Vertex {
            pos: [1.0, 1.0],
            uv: [1.0, 1.0],
        },
        Vertex {
            pos: [0.0, 1.0],
            uv: [0.0, 1.0],
        },
    ]
}

pub fn world_pixel_size() -> glam::Vec2 {
    glam::Vec2::new(
        (MAP_WIDTH * TILE_SIZE) as f32,
        (MAP_HEIGHT * TILE_SIZE) as f32,
    )
}

pub fn build() -> Result<Terrain> {
    let catalog = assets::load_catalog()?;
    let terrain: HashMap<&str, &Sprite> = catalog
        .iter()
        .filter(|sprite| sprite.role == Role::Terrain)
        .map(|sprite| (sprite.id.as_str(), sprite))
        .collect();
    for id in ["grass_a", "grass_b", "dirt", "stone", "sand"] {
        if !terrain.contains_key(id) {
            bail!("manifest is missing terrain sprite `{id}`");
        }
        let sprite = terrain[id];
        if sprite.width != TILE_SIZE || sprite.height != TILE_SIZE {
            bail!(
                "terrain `{id}` must be {TILE_SIZE}×{TILE_SIZE}, got {}×{}",
                sprite.width,
                sprite.height
            );
        }
    }

    let map = build_map();
    let mut sheets: Vec<(String, u32, u32, Vec<u8>)> = Vec::new();
    let mut baked_keys: HashMap<[u8; 5], String> = HashMap::new();
    let mut tile_keys = vec![[0u8; 5]; (MAP_WIDTH * MAP_HEIGHT) as usize];

    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let key = neighborhood(&map, x, y);
            if !baked_keys.contains_key(&key) {
                let id = format!(
                    "t:{}:{}:{}:{}:{}",
                    material_name(key[0]),
                    material_name(key[1]),
                    material_name(key[2]),
                    material_name(key[3]),
                    material_name(key[4])
                );
                let pixels = bake_tile(&terrain, key);
                sheets.push((id.clone(), TILE_SIZE, TILE_SIZE, pixels));
                baked_keys.insert(key, id);
            }
            tile_keys[(y * MAP_WIDTH + x) as usize] = key;
        }
    }

    for sprite in catalog
        .iter()
        .filter(|sprite| sprite.role != Role::Terrain)
    {
        sheets.push((
            sprite.id.clone(),
            sprite.width,
            sprite.height,
            sprite.pixels.clone(),
        ));
    }

    let atlas = assets::pack_atlas(&sheets).context("pack terrain atlas")?;
    let mut instances = Vec::new();

    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let key = tile_keys[(y * MAP_WIDTH + x) as usize];
            let id = baked_keys
                .get(&key)
                .expect("baked tile missing from atlas key map");
            instances.push(instance_from_frame(
                (x * TILE_SIZE) as f32,
                (y * TILE_SIZE) as f32,
                atlas.frames.get(id).with_context(|| format!("atlas `{id}`"))?,
            ));
        }
    }

    place_decals(&atlas, &map, &mut instances)?;
    let mut occluders = Vec::new();
    place_vegetation(&atlas, &map, &mut instances, &mut occluders)?;
    Ok(Terrain {
        atlas,
        instances,
        occluders,
    })
}

fn build_map() -> Vec<u8> {
    let mut map = vec![0u8; (MAP_WIDTH * MAP_HEIGHT) as usize];
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            map[(y * MAP_WIDTH + x) as usize] = material_at(x, y);
        }
    }
    map
}

fn material_at(x: u32, y: u32) -> u8 {
    let fx = x as f32;
    let fy = y as f32;
    let sand_a = ellipse(fx, fy, 50.0, 11.0, 9.0, 6.5);
    let sand_b = ellipse(fx, fy, 14.0, 30.0, 6.5, 4.5);
    if sand_a < 1.0 || sand_b < 1.0 {
        return 4;
    }
    if sand_a < 1.35 || sand_b < 1.4 {
        return 2;
    }

    let path_center = 18.0 + (fx * 0.18).sin() * 5.0;
    let dy = (fy - path_center).abs();
    if dy < 1.35 {
        3
    } else if dy < 2.55 {
        2
    } else if (x + y * 3) % 11 == 0 {
        1
    } else {
        0
    }
}

fn ellipse(x: f32, y: f32, cx: f32, cy: f32, rx: f32, ry: f32) -> f32 {
    let nx = (x - cx) / rx;
    let ny = (y - cy) / ry;
    nx * nx + ny * ny
}

fn neighborhood(map: &[u8], x: u32, y: u32) -> [u8; 5] {
    let center = map[(y * MAP_WIDTH + x) as usize];
    let north = if y == 0 {
        center
    } else {
        map[((y - 1) * MAP_WIDTH + x) as usize]
    };
    let east = if x + 1 >= MAP_WIDTH {
        center
    } else {
        map[(y * MAP_WIDTH + x + 1) as usize]
    };
    let south = if y + 1 >= MAP_HEIGHT {
        center
    } else {
        map[((y + 1) * MAP_WIDTH + x) as usize]
    };
    let west = if x == 0 {
        center
    } else {
        map[(y * MAP_WIDTH + x - 1) as usize]
    };
    [center, north, east, south, west]
}

fn material_name(id: u8) -> &'static str {
    match id {
        0 => "grass_a",
        1 => "grass_b",
        2 => "dirt",
        3 => "stone",
        4 => "sand",
        _ => "grass_a",
    }
}

fn bake_tile(terrain: &HashMap<&str, &Sprite>, key: [u8; 5]) -> Vec<u8> {
    let center = terrain[material_name(key[0])];
    let north = terrain[material_name(key[1])];
    let east = terrain[material_name(key[2])];
    let south = terrain[material_name(key[3])];
    let west = terrain[material_name(key[4])];
    let band = 4i32;
    let mut out = vec![0u8; (TILE_SIZE * TILE_SIZE * 4) as usize];

    for y in 0..TILE_SIZE {
        for x in 0..TILE_SIZE {
            let mut color = assets::sample(center, x, y);
            color = blend_edge(color, north, x, y, key[0], key[1], y as i32, band, 17);
            color = blend_edge(
                color,
                east,
                x,
                y,
                key[0],
                key[2],
                (TILE_SIZE - 1 - x) as i32,
                band,
                19,
            );
            color = blend_edge(
                color,
                south,
                x,
                y,
                key[0],
                key[3],
                (TILE_SIZE - 1 - y) as i32,
                band,
                23,
            );
            color = blend_edge(color, west, x, y, key[0], key[4], x as i32, band, 29);
            let i = ((y * TILE_SIZE + x) * 4) as usize;
            out[i..i + 4].copy_from_slice(&color);
        }
    }
    out
}

fn blend_edge(
    current: [u8; 4],
    neighbor: &Sprite,
    x: u32,
    y: u32,
    center_id: u8,
    neighbor_id: u8,
    dist: i32,
    band: i32,
    salt: u32,
) -> [u8; 4] {
    if center_id == neighbor_id || dist >= band {
        return current;
    }
    let chance = (band - dist) as u32;
    if hash(x / 2, y / 2, salt) % (band as u32) < chance {
        assets::sample(neighbor, x, y)
    } else {
        current
    }
}

fn place_decals(atlas: &Atlas, map: &[u8], instances: &mut Vec<Instance>) -> Result<()> {
    let pebble = atlas
        .frames
        .get("pebble")
        .context("atlas missing `pebble` decal")?;
    let tuft = atlas
        .frames
        .get("tuft")
        .context("atlas missing `tuft` decal")?;

    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let material = map[(y * MAP_WIDTH + x) as usize];
            let ox = (x * TILE_SIZE) as f32;
            let oy = (y * TILE_SIZE) as f32;
            match material {
                0 | 1 if hash(x, y, 101) % 13 == 0 => {
                    instances.push(instance_from_frame(
                        ox + (hash(x, y, 3) % 12) as f32,
                        oy + (hash(x, y, 5) % 10) as f32,
                        tuft,
                    ));
                }
                2 | 3 if hash(x, y, 107) % 11 == 0 => {
                    instances.push(instance_from_frame(
                        ox + (hash(x, y, 7) % 14) as f32,
                        oy + (hash(x, y, 11) % 14) as f32,
                        pebble,
                    ));
                }
                4 if hash(x, y, 109) % 17 == 0 => {
                    instances.push(instance_from_frame(
                        ox + (hash(x, y, 13) % 14) as f32,
                        oy + (hash(x, y, 17) % 14) as f32,
                        pebble,
                    ));
                }
                _ => {}
            }
        }
    }
    Ok(())
}

fn place_vegetation(
    atlas: &Atlas,
    map: &[u8],
    instances: &mut Vec<Instance>,
    occluders: &mut Vec<u32>,
) -> Result<()> {
    let world_w = (MAP_WIDTH * TILE_SIZE) as f32;
    let world_h = (MAP_HEIGHT * TILE_SIZE) as f32;
    let mut occupied: Vec<(f32, f32, f32)> = Vec::new();

    scatter(
        atlas,
        map,
        instances,
        occluders,
        &mut occupied,
        &["clump_a", "flower_a", "flower_b", "mushroom_a"],
        14.0,
        10.0,
        0.55,
        world_w,
        world_h,
        201,
        false,
    )?;
    scatter(
        atlas,
        map,
        instances,
        occluders,
        &mut occupied,
        &["fern_a", "fern_b"],
        28.0,
        18.0,
        0.28,
        world_w,
        world_h,
        207,
        false,
    )?;
    scatter(
        atlas,
        map,
        instances,
        occluders,
        &mut occupied,
        &["bush_c", "bush_b", "bush_a"],
        28.0,
        22.0,
        0.38,
        world_w,
        world_h,
        211,
        true,
    )?;
    scatter(
        atlas,
        map,
        instances,
        occluders,
        &mut occupied,
        &["tree_c", "tree_b", "tree_a"],
        58.0,
        48.0,
        0.28,
        world_w,
        world_h,
        223,
        true,
    )?;
    Ok(())
}

fn scatter(
    atlas: &Atlas,
    map: &[u8],
    instances: &mut Vec<Instance>,
    occluders: &mut Vec<u32>,
    occupied: &mut Vec<(f32, f32, f32)>,
    ids: &[&str],
    step: f32,
    radius: f32,
    chance: f32,
    world_w: f32,
    world_h: f32,
    salt: u32,
    occludes: bool,
) -> Result<()> {
    let frames: Vec<&Frame> = ids
        .iter()
        .map(|id| {
            atlas
                .frames
                .get(*id)
                .with_context(|| format!("atlas missing prop `{id}`"))
        })
        .collect::<Result<Vec<_>>>()?;

    let cols = ((world_w / step).ceil() as u32).max(1);
    let rows = ((world_h / step).ceil() as u32).max(1);
    for gy in 0..rows {
        for gx in 0..cols {
            let jx = (hash(gx, gy, salt) % step as u32) as f32;
            let jy = (hash(gx, gy, salt + 7) % step as u32) as f32;
            let x = gx as f32 * step + jx - step * 0.25;
            let y = gy as f32 * step + jy - step * 0.25;
            if x < 8.0 || y < 8.0 || x > world_w - 24.0 || y > world_h - 24.0 {
                continue;
            }
            let density = density_at(map, x, y);
            if density <= 0.02 {
                continue;
            }
            let grove = value_noise(x / 92.0, y / 92.0, salt);
            let roll = (hash(gx, gy, salt + 13) % 1000) as f32 / 1000.0;
            if roll > chance * density * (0.45 + grove * 0.7) {
                continue;
            }
            if too_close(occupied, x, y, radius) {
                continue;
            }
            if path_clearance(x, y) < 1.7 && radius > 20.0 {
                continue;
            }
            let variant = hash(gx, gy, salt + 29) as usize % frames.len();
            let frame = frames[variant];
            let px = (x - frame.w as f32 * 0.5).clamp(0.0, world_w - frame.w as f32);
            let py = (y - frame.h as f32 * 0.5).clamp(0.0, world_h - frame.h as f32);
            if occludes {
                occluders.push(instances.len() as u32);
            }
            instances.push(instance_from_frame(px, py, frame));
            occupied.push((x, y, radius * 0.72));
        }
    }
    Ok(())
}

fn density_at(map: &[u8], x: f32, y: f32) -> f32 {
    let tx = (x / TILE_SIZE as f32).floor().clamp(0.0, (MAP_WIDTH - 1) as f32) as u32;
    let ty = (y / TILE_SIZE as f32).floor().clamp(0.0, (MAP_HEIGHT - 1) as f32) as u32;
    match map[(ty * MAP_WIDTH + tx) as usize] {
        0 | 1 => 1.0,
        2 => 0.28,
        3 => 0.04,
        4 => 0.12,
        _ => 0.0,
    }
}

fn path_clearance(x: f32, y: f32) -> f32 {
    let tx = x / TILE_SIZE as f32;
    let ty = y / TILE_SIZE as f32;
    let path_center = 18.0 + (tx * 0.18).sin() * 5.0;
    (ty - path_center).abs()
}

fn too_close(occupied: &[(f32, f32, f32)], x: f32, y: f32, radius: f32) -> bool {
    occupied.iter().any(|(ox, oy, r)| {
        let dx = x - ox;
        let dy = y - oy;
        dx * dx + dy * dy < (radius.min(*r)).powi(2)
    })
}

fn value_noise(x: f32, y: f32, salt: u32) -> f32 {
    let x0 = x.floor();
    let y0 = y.floor();
    let fx = x - x0;
    let fy = y - y0;
    let ix = x0 as u32;
    let iy = y0 as u32;
    let v00 = (hash(ix, iy, salt) % 1000) as f32 / 1000.0;
    let v10 = (hash(ix.wrapping_add(1), iy, salt) % 1000) as f32 / 1000.0;
    let v01 = (hash(ix, iy.wrapping_add(1), salt) % 1000) as f32 / 1000.0;
    let v11 = (hash(ix.wrapping_add(1), iy.wrapping_add(1), salt) % 1000) as f32 / 1000.0;
    let sx = fx * fx * (3.0 - 2.0 * fx);
    let sy = fy * fy * (3.0 - 2.0 * fy);
    let a = v00 + (v10 - v00) * sx;
    let b = v01 + (v11 - v01) * sx;
    a + (b - a) * sy
}

pub fn update_cover_fade(instances: &mut [Instance], occluders: &[u32], probe: glam::Vec2) {
    const GHOST: f32 = 0.32;
    for &index in occluders {
        let Some(instance) = instances.get_mut(index as usize) else {
            continue;
        };
        let cx = instance.world_xy[0] + instance.size_xy[0] * 0.5;
        let cy = instance.world_xy[1] + instance.size_xy[1] * 0.5;
        let rx = instance.size_xy[0] * 0.34;
        let ry = instance.size_xy[1] * 0.34;
        let nx = (probe.x - cx) / rx.max(1.0);
        let ny = (probe.y - cy) / ry.max(1.0);
        let target = if nx * nx + ny * ny <= 1.0 { GHOST } else { 1.0 };
        instance.alpha += (target - instance.alpha) * 0.22;
    }
}

fn instance_from_frame(world_x: f32, world_y: f32, frame: &Frame) -> Instance {
    Instance {
        world_xy: [world_x, world_y],
        atlas_xy: [frame.x as f32, frame.y as f32],
        size_xy: [frame.w as f32, frame.h as f32],
        alpha: 1.0,
        _pad: 0.0,
    }
}

fn hash(x: u32, y: u32, salt: u32) -> u32 {
    let mut n = x.wrapping_mul(374761393) ^ y.wrapping_mul(668265263) ^ salt;
    n = (n ^ (n >> 13)).wrapping_mul(1274126177);
    n ^ (n >> 16)
}
