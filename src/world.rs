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
}

pub struct Terrain {
    pub atlas: Atlas,
    pub instances: Vec<Instance>,
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

    for sprite in catalog.iter().filter(|sprite| sprite.role == Role::Decal) {
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
    Ok(Terrain { atlas, instances })
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
    let band = 7i32;
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
    if hash(x, y, salt) % (band as u32) < chance {
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

fn instance_from_frame(world_x: f32, world_y: f32, frame: &Frame) -> Instance {
    Instance {
        world_xy: [world_x, world_y],
        atlas_xy: [frame.x as f32, frame.y as f32],
        size_xy: [frame.w as f32, frame.h as f32],
    }
}

fn hash(x: u32, y: u32, salt: u32) -> u32 {
    let mut n = x.wrapping_mul(374761393) ^ y.wrapping_mul(668265263) ^ salt;
    n = (n ^ (n >> 13)).wrapping_mul(1274126177);
    n ^ (n >> 16)
}
