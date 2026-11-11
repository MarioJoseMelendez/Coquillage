use std::collections::HashMap;

use anyhow::{bail, Context, Result};
use bytemuck::{Pod, Zeroable};

use crate::assets::{self, Atlas, Frame, Role, Sprite};

pub const TILE_SIZE: u32 = 32;
pub const CELL_SIZE: u32 = 16;
pub const LOGICAL_WIDTH: f32 = 640.0;
pub const LOGICAL_HEIGHT: f32 = 360.0;
pub const MAP_WIDTH: u32 = 64;
pub const MAP_HEIGHT: u32 = 40;
pub const GRID_WIDTH: u32 = MAP_WIDTH * 2;
pub const GRID_HEIGHT: u32 = MAP_HEIGHT * 2;

const G_GRASS: u8 = 0;
const G_DIRT: u8 = 1;
const G_STONE: u8 = 2;
const G_SAND: u8 = 3;
const G_WATER: u8 = 4;
const G_TAKEN: u8 = 5;

/// Interaction layers on the 16×16 grid. Pure top-down stacks the CPU owns.
/// Underground comes later; do not allocate it yet.
/// 0 ground, 1 floor props, 2 cover (trees / bushes / tall plants).
#[allow(dead_code)]
pub const LAYER_GROUND: u8 = 0;
#[allow(dead_code)]
pub const LAYER_FLOOR: u8 = 1;
#[allow(dead_code)]
pub const LAYER_COVER: u8 = 2;
pub const MASK_PAD_WIDTH: u32 = 256;

pub struct Layers {
    pub ground: Vec<u8>,
    pub floor: Vec<u8>,
    pub cover: Vec<u8>,
    pub solid: Vec<u8>,
}

impl Layers {
    fn from_ground(ground: Vec<u8>) -> Self {
        let n = ground.len();
        Self {
            ground,
            floor: vec![0; n],
            cover: vec![0; n],
            solid: vec![0; n],
        }
    }

    fn mark_floor(&mut self, cx: u32, cy: u32, ow: u32, oh: u32, kind: u8) {
        let x1 = (cx + ow).min(GRID_WIDTH);
        let y1 = (cy + oh).min(GRID_HEIGHT);
        for y in cy..y1 {
            for x in cx..x1 {
                self.floor[(y * GRID_WIDTH + x) as usize] = kind;
            }
        }
    }

    fn mark_cover(&mut self, cx: u32, cy: u32, ow: u32, oh: u32) {
        let x1 = (cx + ow).min(GRID_WIDTH);
        let y1 = (cy + oh).min(GRID_HEIGHT);
        for y in cy..y1 {
            for x in cx..x1 {
                self.cover[(y * GRID_WIDTH + x) as usize] = 1;
            }
        }
    }

    fn mark_solid(&mut self, cx: u32, cy: u32, ow: u32, oh: u32) {
        let x1 = (cx + ow).min(GRID_WIDTH);
        let y1 = (cy + oh).min(GRID_HEIGHT);
        for y in cy..y1 {
            for x in cx..x1 {
                self.solid[(y * GRID_WIDTH + x) as usize] = 1;
            }
        }
    }

    /// Water and tree trunks block. Floor plants and bushes do not.
    pub fn blocked_aabb(&self, x: f32, y: f32, w: f32, h: f32) -> bool {
        let x0 = (x / CELL_SIZE as f32).floor() as i32;
        let y0 = (y / CELL_SIZE as f32).floor() as i32;
        let x1 = ((x + w - 0.001) / CELL_SIZE as f32).floor() as i32;
        let y1 = ((y + h - 0.001) / CELL_SIZE as f32).floor() as i32;
        for cy in y0..=y1 {
            for cx in x0..=x1 {
                if cx < 0 || cy < 0 || cx >= GRID_WIDTH as i32 || cy >= GRID_HEIGHT as i32 {
                    return true;
                }
                let i = (cy as u32 * GRID_WIDTH + cx as u32) as usize;
                if self.ground[i] == G_WATER || self.solid[i] != 0 {
                    return true;
                }
            }
        }
        false
    }

    fn ground_at(&self, x: f32, y: f32) -> Option<u8> {
        let cx = (x / CELL_SIZE as f32).floor() as i32;
        let cy = (y / CELL_SIZE as f32).floor() as i32;
        if cx < 0 || cy < 0 || cx >= GRID_WIDTH as i32 || cy >= GRID_HEIGHT as i32 {
            return None;
        }
        Some(self.ground[(cy as u32 * GRID_WIDTH + cx as u32) as usize])
    }

    pub fn spawn_point(&self, prefer: glam::Vec2, cover: &[Instance]) -> glam::Vec2 {
        const HIT_W: f32 = 10.0;
        const HIT_H: f32 = 8.0;
        const VIS: f32 = 32.0;
        let under = |pos: glam::Vec2| {
            let c = pos + glam::Vec2::splat(VIS * 0.5);
            cover.iter().any(|inst| {
                c.x >= inst.world_xy[0] + 4.0
                    && c.x <= inst.world_xy[0] + inst.size_xy[0] - 4.0
                    && c.y >= inst.world_xy[1] + 4.0
                    && c.y <= inst.world_xy[1] + inst.size_xy[1] - 6.0
            })
        };
        let mut best_open = None;
        for r in 0..140 {
            for dy in -(r as i32)..=r as i32 {
                for dx in -(r as i32)..=r as i32 {
                    if dx.abs() != r as i32 && dy.abs() != r as i32 {
                        continue;
                    }
                    let pos = glam::Vec2::new(
                        prefer.x + dx as f32 * CELL_SIZE as f32 - VIS * 0.5,
                        prefer.y + dy as f32 * CELL_SIZE as f32 - VIS * 0.5,
                    );
                    let hx = pos.x + (VIS - HIT_W) * 0.5;
                    let hy = pos.y + VIS - HIT_H - 1.0;
                    if self.blocked_aabb(hx, hy, HIT_W, HIT_H) || under(pos) {
                        continue;
                    }
                    let kind = self.ground_at(pos.x + VIS * 0.5, pos.y + VIS * 0.75);
                    if matches!(kind, Some(G_DIRT) | Some(G_SAND)) {
                        return pos;
                    }
                    if best_open.is_none() {
                        best_open = Some(pos);
                    }
                }
            }
        }
        best_open.unwrap_or(prefer)
    }

    /// R8 mask, rows padded to 256. 0 = rain hits ground, 255 = covered or water.
    pub fn rain_mask(&self) -> Vec<u8> {
        let mut out = vec![0u8; (MASK_PAD_WIDTH * GRID_HEIGHT) as usize];
        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                let i = (y * GRID_WIDTH + x) as usize;
                let blocked = self.ground[i] == G_WATER || self.cover[i] != 0;
                out[(y * MASK_PAD_WIDTH + x) as usize] = if blocked { 255 } else { 0 };
            }
        }
        out
    }
}

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
    pub atlas_wh: [f32; 2],
    pub alpha: f32,
    pub flags: f32,
}

pub struct Terrain {
    pub atlas: Atlas,
    pub instances: Vec<Instance>,
    #[allow(dead_code)]
    pub occluders: Vec<u32>,
    pub casters: Vec<ShadowCaster>,
    pub layers: Layers,
    pub rain_mask: Vec<u8>,
    pub shadow_start: u32,
    pub shadow_count: u32,
    pub cover_start: u32,
    pub player_caster: u32,
}

pub struct ShadowCaster {
    pub shadow_index: u32,
    pub origin: [f32; 2],
    pub height: f32,
    pub base_size: [f32; 2],
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
    for id in [
        "grass_a",
        "grass_b",
        "dirt",
        "stone",
        "sand",
        "water_deep",
        "water_shallow",
    ] {
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
    let mut layers = Layers::from_ground(build_grid(&map));
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

    let mut pond_ids = vec![String::new(); (MAP_WIDTH * MAP_HEIGHT) as usize];
    let mut water_ids: Vec<Option<String>> = vec![None; (MAP_WIDTH * MAP_HEIGHT) as usize];
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let idx = (y * MAP_WIDTH + x) as usize;
            let key = tile_keys[idx];
            let land_id = baked_keys
                .get(&key)
                .expect("baked tile missing from atlas key map")
                .clone();
            let (has_sand, has_water) = tile_has_shore(&layers.ground, x, y);
            if !has_sand && !has_water {
                pond_ids[idx] = land_id;
                continue;
            }
            let id = format!("p:{x}:{y}");
            let mut pixels = bake_tile(&terrain, key);
            paint_occupancy(&mut pixels, &terrain, &layers.ground, x, y, false);
            sheets.push((id.clone(), TILE_SIZE, TILE_SIZE, pixels));
            pond_ids[idx] = id;
            if has_water {
                let wid = format!("w:{x}:{y}");
                let mut overlay = vec![0u8; (TILE_SIZE * TILE_SIZE * 4) as usize];
                paint_occupancy(&mut overlay, &terrain, &layers.ground, x, y, true);
                sheets.push((wid.clone(), TILE_SIZE, TILE_SIZE, overlay));
                water_ids[idx] = Some(wid);
            }
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
            let idx = (y * MAP_WIDTH + x) as usize;
            let id = &pond_ids[idx];
            instances.push(instance_from_frame(
                (x * TILE_SIZE) as f32,
                (y * TILE_SIZE) as f32,
                atlas.frames.get(id).with_context(|| format!("atlas `{id}`"))?,
            ));
        }
    }

    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let Some(id) = &water_ids[(y * MAP_WIDTH + x) as usize] else {
                continue;
            };
            let frame = atlas
                .frames
                .get(id)
                .with_context(|| format!("atlas `{id}`"))?;
            let mut inst = instance_from_frame(
                (x * TILE_SIZE) as f32,
                (y * TILE_SIZE) as f32,
                frame,
            );
            inst.flags = 1.0;
            instances.push(inst);
        }
    }

    place_decals(&atlas, &mut layers, &mut instances)?;
    let mut occluders = Vec::new();
    let mut casters = Vec::new();
    let (shadow_start, shadow_count, cover_start, player_caster) =
        place_occupants(&atlas, &mut layers, &mut instances, &mut occluders, &mut casters)?;
    let rain_mask = layers.rain_mask();
    Ok(Terrain {
        atlas,
        instances,
        occluders,
        casters,
        layers,
        rain_mask,
        shadow_start,
        shadow_count,
        cover_start,
        player_caster,
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

fn build_grid(map: &[u8]) -> Vec<u8> {
    let mut grid = vec![G_GRASS; (GRID_WIDTH * GRID_HEIGHT) as usize];
    for cy in 0..GRID_HEIGHT {
        for cx in 0..GRID_WIDTH {
            let tx = cx / 2;
            let ty = cy / 2;
            let land = map[(ty * MAP_WIDTH + tx) as usize];
            let i = (cy * GRID_WIDTH + cx) as usize;
            grid[i] = match land {
                3 => G_STONE,
                2 => G_DIRT,
                _ => G_GRASS,
            };
            if grid[i] != G_STONE && water_t(cell_center(cx), cell_center(cy)) > 0.18 {
                grid[i] = G_WATER;
            }
        }
    }
    // Light inner bite so the water/sand lip is jagged, not a fat swallow.
    for cy in 0..GRID_HEIGHT {
        for cx in 0..GRID_WIDTH {
            let i = (cy * GRID_WIDTH + cx) as usize;
            if grid[i] != G_WATER
                || (!neighbor_cardinal(&grid, cx, cy, G_GRASS)
                    && !neighbor_cardinal(&grid, cx, cy, G_DIRT))
            {
                continue;
            }
            let nibble = value_noise(cx as f32 / 4.0, cy as f32 / 4.0, 67);
            if nibble > 0.72 {
                grid[i] = G_SAND;
            }
        }
    }
    let water = grid.clone();
    let mut sand = grid;
    for cy in 0..GRID_HEIGHT {
        for cx in 0..GRID_WIDTH {
            let i = (cy * GRID_WIDTH + cx) as usize;
            if sand[i] != G_GRASS && sand[i] != G_DIRT {
                continue;
            }
            let d = dist_to(&water, cx, cy, G_WATER, 3);
            if d > 3 {
                continue;
            }
            let edge = value_noise(cx as f32 / 3.2, cy as f32 / 3.2, 91);
            let bump = value_noise(cx as f32 / 6.0, cy as f32 / 6.0, 77);
            // Always 1 cell, almost always a second; third only as a small bump.
            let beach = d == 1 || (d == 2 && edge > 0.18) || (d == 3 && bump > 0.78 && edge > 0.55);
            if beach {
                sand[i] = G_SAND;
            }
        }
    }
    // Close grass pockets trapped inside the beach.
    for _ in 0..2 {
        let snap = sand.clone();
        for cy in 0..GRID_HEIGHT {
            for cx in 0..GRID_WIDTH {
                let i = (cy * GRID_WIDTH + cx) as usize;
                if snap[i] != G_GRASS && snap[i] != G_DIRT {
                    continue;
                }
                if count_kind(&snap, cx, cy, G_SAND) >= 5 {
                    sand[i] = G_SAND;
                }
            }
        }
    }
    sand
}

fn count_kind(grid: &[u8], cx: u32, cy: u32, kind: u8) -> u32 {
    let mut n = 0u32;
    for dy in -1i32..=1 {
        for dx in -1i32..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }
            if cell_at(grid, cx as i32 + dx, cy as i32 + dy) == Some(kind) {
                n += 1;
            }
        }
    }
    n
}

fn neighbor_cardinal(grid: &[u8], cx: u32, cy: u32, kind: u8) -> bool {
    const DIRS: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];
    for (dx, dy) in DIRS {
        if cell_at(grid, cx as i32 + dx, cy as i32 + dy) == Some(kind) {
            return true;
        }
    }
    false
}

fn dist_to(grid: &[u8], cx: u32, cy: u32, kind: u8, maxd: i32) -> u32 {
    let mut best = (maxd + 1) as u32;
    for dy in -maxd..=maxd {
        for dx in -maxd..=maxd {
            let cheb = dx.abs().max(dy.abs()) as u32;
            if cheb == 0 || cheb >= best {
                continue;
            }
            if cell_at(grid, cx as i32 + dx, cy as i32 + dy) == Some(kind) {
                best = cheb;
            }
        }
    }
    best
}

fn neighbor_is(grid: &[u8], cx: u32, cy: u32, kind: u8) -> bool {
    for dy in -1i32..=1 {
        for dx in -1i32..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }
            let nx = cx as i32 + dx;
            let ny = cy as i32 + dy;
            if nx < 0 || ny < 0 || nx >= GRID_WIDTH as i32 || ny >= GRID_HEIGHT as i32 {
                continue;
            }
            if grid[(ny as u32 * GRID_WIDTH + nx as u32) as usize] == kind {
                return true;
            }
        }
    }
    false
}

fn cell_center(c: u32) -> f32 {
    (c * CELL_SIZE + CELL_SIZE / 2) as f32
}

fn cell_at(grid: &[u8], cx: i32, cy: i32) -> Option<u8> {
    if cx < 0 || cy < 0 || cx >= GRID_WIDTH as i32 || cy >= GRID_HEIGHT as i32 {
        return None;
    }
    Some(grid[(cy as u32 * GRID_WIDTH + cx as u32) as usize])
}

fn tile_has_shore(grid: &[u8], tx: u32, ty: u32) -> (bool, bool) {
    let mut sand = false;
    let mut water = false;
    for oy in 0..2u32 {
        for ox in 0..2u32 {
            match grid[((ty * 2 + oy) * GRID_WIDTH + tx * 2 + ox) as usize] {
                G_SAND => sand = true,
                G_WATER => water = true,
                _ => {}
            }
        }
    }
    (sand, water)
}

fn paint_occupancy(
    out: &mut [u8],
    terrain: &HashMap<&str, &Sprite>,
    grid: &[u8],
    tx: u32,
    ty: u32,
    overlay: bool,
) {
    let sand = terrain["sand"];
    for y in 0..TILE_SIZE {
        for x in 0..TILE_SIZE {
            let cx = tx * 2 + x / CELL_SIZE;
            let cy = ty * 2 + y / CELL_SIZE;
            let kind = grid[(cy * GRID_WIDTH + cx) as usize];
            let i = ((y * TILE_SIZE + x) * 4) as usize;
            if overlay {
                if kind == G_WATER {
                    out[i..i + 4].copy_from_slice(&water_cell_pixel(grid, cx, cy, x, y));
                } else {
                    out[i..i + 4].copy_from_slice(&[0, 0, 0, 0]);
                }
                continue;
            }
            match kind {
                G_SAND => {
                    let mut c = assets::sample(sand, x, y);
                    if neighbor_cardinal(grid, cx, cy, G_WATER) {
                        let sx = (x % CELL_SIZE) as i32;
                        let sy = (y % CELL_SIZE) as i32;
                        let toward_water = (sx < 4
                            && cell_at(grid, cx as i32 - 1, cy as i32) == Some(G_WATER))
                            || (sx >= CELL_SIZE as i32 - 4
                                && cell_at(grid, cx as i32 + 1, cy as i32) == Some(G_WATER))
                            || (sy < 4
                                && cell_at(grid, cx as i32, cy as i32 - 1) == Some(G_WATER))
                            || (sy >= CELL_SIZE as i32 - 4
                                && cell_at(grid, cx as i32, cy as i32 + 1) == Some(G_WATER));
                        if toward_water && hash(cx * CELL_SIZE + x, cy * CELL_SIZE + y, 41) % 5 != 0
                        {
                            c = if hash(x, y, 47) % 3 == 0 {
                                FOAM
                            } else {
                                FOAM_DK
                            };
                        }
                    }
                    out[i..i + 4].copy_from_slice(&c);
                }
                G_WATER => {
                    out[i..i + 4].copy_from_slice(&water_cell_pixel(grid, cx, cy, x, y));
                }
                _ => {}
            }
        }
    }
}

fn water_cell_pixel(grid: &[u8], cx: u32, cy: u32, lx: u32, ly: u32) -> [u8; 4] {
    let wx = cx * CELL_SIZE + lx % CELL_SIZE;
    let wy = cy * CELL_SIZE + ly % CELL_SIZE;
    let t = water_t(cell_center(cx), cell_center(cy));
    let depth = ((t - 0.18) / 0.55).clamp(0.0, 1.0);
    let mut c = water_fill(wx, wy, depth);
    let sx = (lx % CELL_SIZE) as i32;
    let sy = (ly % CELL_SIZE) as i32;
    let mut edge = 99i32;
    if cell_at(grid, cx as i32 - 1, cy as i32) == Some(G_SAND) {
        edge = edge.min(sx);
    }
    if cell_at(grid, cx as i32 + 1, cy as i32) == Some(G_SAND) {
        edge = edge.min(CELL_SIZE as i32 - 1 - sx);
    }
    if cell_at(grid, cx as i32, cy as i32 - 1) == Some(G_SAND) {
        edge = edge.min(sy);
    }
    if cell_at(grid, cx as i32, cy as i32 + 1) == Some(G_SAND) {
        edge = edge.min(CELL_SIZE as i32 - 1 - sy);
    }
    let near = neighbor_is(grid, cx, cy, G_SAND);
    let jag = 4 + (hash(wx / 2, wy / 2, 17) % 4) as i32;
    let foam = (edge < jag && hash(wx, wy, 23) % 6 != 0)
        || (near && edge < jag + 3 && hash(wx, wy, 31) % 5 == 0)
        || (near && hash(wx, wy, 43) % 17 == 0);
    if foam {
        c = match hash(wx, wy, 29) % 5 {
            0 => WATER_HI,
            1 => FOAM_DK,
            _ => FOAM,
        };
    }
    c
}

fn material_at(x: u32, y: u32) -> u8 {
    let fx = x as f32;
    let fy = y as f32;
    let path_center = 18.0 + (fx * 0.18).sin() * 5.0;
    let dy = (fy - path_center).abs();
    if dy < 1.35 {
        3
    } else if dy < 2.55 {
        2
    } else {
        0
    }
}

const WATER_HI: [u8; 4] = [168, 206, 186, 255];
const WATER_MID: [u8; 4] = [46, 112, 120, 255];
const WATER_DK: [u8; 4] = [28, 62, 82, 255];
const WATER_SH: [u8; 4] = [44, 36, 78, 255];
const FOAM: [u8; 4] = [236, 238, 224, 255];
const FOAM_DK: [u8; 4] = [198, 210, 196, 255];

fn water_t(wx: f32, wy: f32) -> f32 {
    pond_t(wx, wy, 768.0, 512.0, 262.0, 179.0, 61)
        .max(pond_t(wx, wy, 1600.0, 352.0, 269.0, 186.0, 73))
        .max(pond_t(wx, wy, 448.0, 960.0, 205.0, 144.0, 89))
}

fn pond_t(wx: f32, wy: f32, cx: f32, cy: f32, rx: f32, ry: f32, salt: u32) -> f32 {
    let n = value_noise(wx / 32.0, wy / 32.0, salt) - 0.5;
    let n2 = value_noise(wx / 16.0, wy / 16.0, salt.wrapping_add(11)) - 0.5;
    let ang = (wy - cy).atan2(wx - cx);
    let jag = 0.12 * n * 2.0 + 0.07 * n2 * 2.0 + 0.05 * (ang * 3.0 + salt as f32 * 0.07).sin();
    let d = ellipse(
        wx,
        wy,
        cx,
        cy,
        (rx * (1.0 + jag)).max(32.0),
        (ry * (1.0 + jag * 0.82)).max(24.0),
    );
    1.0 - d
}

fn water_fill(wx: u32, wy: u32, depth: f32) -> [u8; 4] {
    // World-space clumps, not a 32px stamp of five ellipses.
    let d = depth * 0.40;
    let mid = [
        (WATER_MID[0] as f32 * (1.0 - d) + WATER_DK[0] as f32 * d) as u8,
        (WATER_MID[1] as f32 * (1.0 - d) + WATER_DK[1] as f32 * d) as u8,
        (WATER_MID[2] as f32 * (1.0 - d) + WATER_DK[2] as f32 * d) as u8,
        255,
    ];
    let n = hash(wx, wy, 91) % 1000;
    let clump = hash(wx / 3, wy / 5, 44) % 31;
    let clump_b = hash(wx / 5, wy / 3, 71) % 37;
    if clump == 0 && hash(wx, wy, 3) % 4 < 2 {
        WATER_DK
    } else if clump_b == 0 && hash(wx, wy, 9) % 5 == 0 {
        WATER_SH
    } else if n < 35 {
        [
            (WATER_HI[0] as f32 * (1.0 - d * 0.5) + mid[0] as f32 * (d * 0.5)) as u8,
            (WATER_HI[1] as f32 * (1.0 - d * 0.5) + mid[1] as f32 * (d * 0.5)) as u8,
            (WATER_HI[2] as f32 * (1.0 - d * 0.5) + mid[2] as f32 * (d * 0.5)) as u8,
            255,
        ]
    } else if n < 140 {
        [
            mid[0].saturating_sub(8),
            mid[1].saturating_sub(6),
            mid[2].saturating_add(4),
            255,
        ]
    } else {
        mid
    }
}

fn is_water(id: u8) -> bool {
    matches!(id, 5 | 6)
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

fn same_family(a: u8, b: u8) -> bool {
    a == b
        || matches!((a, b), (0, 1) | (1, 0) | (5, 6) | (6, 5))
}

fn material_name(id: u8) -> &'static str {
    match id {
        0 => "grass_a",
        1 => "grass_b",
        2 => "dirt",
        3 => "stone",
        4 => "sand",
        5 => "water_deep",
        6 => "water_shallow",
        _ => "grass_a",
    }
}

fn bake_tile(terrain: &HashMap<&str, &Sprite>, key: [u8; 5]) -> Vec<u8> {
    let center = terrain[material_name(key[0])];
    let north = terrain[material_name(key[1])];
    let east = terrain[material_name(key[2])];
    let south = terrain[material_name(key[3])];
    let west = terrain[material_name(key[4])];
    let band = 2i32;
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
    if same_family(center_id, neighbor_id) || dist >= band {
        return current;
    }
    let chance = (band - dist) as u32;
    if hash(x / 2, y / 2, salt) % (band as u32) >= chance {
        return current;
    }
    // Water keeps its body; land only leaves a foam lip, not grass eating the pond.
    if is_water(center_id) && !is_water(neighbor_id) {
        return if hash(x, y, salt.wrapping_add(41)) % 3 == 0 {
            [214, 224, 210, 255]
        } else {
            [168, 206, 186, 255]
        };
    }
    assets::sample(neighbor, x, y)
}

fn place_decals(atlas: &Atlas, layers: &mut Layers, instances: &mut Vec<Instance>) -> Result<()> {
    let pebble = atlas
        .frames
        .get("pebble")
        .context("atlas missing `pebble` decal")?;

    for cy in 0..GRID_HEIGHT {
        for cx in 0..GRID_WIDTH {
            let kind = layers.ground[(cy * GRID_WIDTH + cx) as usize];
            if !matches!(kind, G_DIRT | G_STONE | G_SAND) {
                continue;
            }
            if hash(cx, cy, 107) % 14 != 0 {
                continue;
            }
            instances.push(instance_from_frame(
                (cx * CELL_SIZE) as f32,
                (cy * CELL_SIZE) as f32,
                pebble,
            ));
            layers.mark_floor(cx, cy, 1, 1, 3);
        }
    }
    Ok(())
}

fn place_occupants(
    atlas: &Atlas,
    layers: &mut Layers,
    instances: &mut Vec<Instance>,
    occluders: &mut Vec<u32>,
    casters: &mut Vec<ShadowCaster>,
) -> Result<(u32, u32, u32, u32)> {
    let shadow = atlas.frames.get("shadow");
    let shadow_sm = atlas.frames.get("shadow_sm");
    let mut blobs = Vec::new();
    let mut floor = Vec::new();
    let mut cover = Vec::new();
    let mut cover_rank = Vec::new();

    // Cover first so occupancy is claimed. Draw order is rebuilt at the end:
    // shadows → floor plants → cover (taller last). Grass never paints over trees.
    place_kind(
        atlas, layers, &mut cover, Some((&mut blobs, casters)), Some(&mut cover_rank),
        &["tree_a"], 4, 4, 0.014, 223, 48.0, shadow, 4.0, true, 0,
    )?;
    place_kind(
        atlas, layers, &mut cover, Some((&mut blobs, casters)), Some(&mut cover_rank),
        &["tree_b"], 3, 3, 0.018, 229, 42.0, shadow, 3.6, true, 0,
    )?;
    place_kind(
        atlas, layers, &mut cover, Some((&mut blobs, casters)), Some(&mut cover_rank),
        &["tree_c"], 3, 3, 0.024, 233, 30.0, shadow, 3.2, true, 0,
    )?;
    place_kind(
        atlas, layers, &mut cover, Some((&mut blobs, casters)), Some(&mut cover_rank),
        &["bush_a", "bush_b", "bush_c"], 2, 2, 0.06, 241, 24.0, shadow_sm, 2.8, true, 0,
    )?;
    place_kind(
        atlas, layers, &mut cover, Some((&mut blobs, casters)), Some(&mut cover_rank),
        &["fern_a", "fern_b"], 2, 2, 0.035, 251, 16.0, shadow_sm, 2.7, true, 0,
    )?;
    place_kind(
        atlas, layers, &mut cover, Some((&mut blobs, casters)), Some(&mut cover_rank),
        &["grass_tall", "grass_tall_b"], 2, 2, 0.22, 257, 20.0, shadow_sm, 2.5, true, 0,
    )?;
    place_kind(
        atlas, layers, &mut floor, None, None,
        &["grass_short", "grass_short_b"], 1, 1, 0.22, 263, 0.0, None, 2.2, false, 1,
    )?;
    place_kind(
        atlas, layers, &mut floor, None, None,
        &["flower_a", "flower_b"], 1, 1, 0.08, 269, 0.0, None, 2.1, false, 2,
    )?;
    place_kind(
        atlas, layers, &mut floor, None, None,
        &["mushroom_a"], 1, 1, 0.025, 271, 0.0, None, 0.0, false, 4,
    )?;

    let mut order: Vec<usize> = (0..cover.len()).collect();
    order.sort_by(|&i, &j| {
        cover_rank[i]
            .0
            .partial_cmp(&cover_rank[j].0)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(
                cover_rank[i]
                    .1
                    .partial_cmp(&cover_rank[j].1)
                    .unwrap_or(std::cmp::Ordering::Equal),
            )
    });
    let cover: Vec<Instance> = order.iter().map(|&i| cover[i]).collect();

    if let Some(sm) = shadow_sm {
        push_shadow(&mut blobs, casters, sm, 0.0, 0.0, 12.0);
    }
    let player_caster = casters.len().saturating_sub(1) as u32;
    let shadow_start = instances.len() as u32;
    let shadow_count = blobs.len() as u32;
    for caster in casters.iter_mut() {
        caster.shadow_index += shadow_start;
    }
    instances.append(&mut blobs);
    instances.append(&mut floor);
    let cover_start = instances.len() as u32;
    let cover_end = cover_start + cover.len() as u32;
    instances.extend(cover);
    occluders.extend(cover_start..cover_end);
    Ok((shadow_start, shadow_count, cover_start, player_caster))
}

fn place_kind(
    atlas: &Atlas,
    layers: &mut Layers,
    sprites: &mut Vec<Instance>,
    mut shadow_pass: Option<(&mut Vec<Instance>, &mut Vec<ShadowCaster>)>,
    mut rank: Option<&mut Vec<(f32, f32)>>,
    ids: &[&str],
    ow: u32,
    oh: u32,
    chance: f32,
    salt: u32,
    height: f32,
    shadow: Option<&Frame>,
    wind: f32,
    covers: bool,
    floor_kind: u8,
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

    for cy in 1..GRID_HEIGHT.saturating_sub(oh.max(1)) {
        for cx in 1..GRID_WIDTH.saturating_sub(ow.max(1)) {
            if !rect_is(&layers.ground, cx, cy, ow, oh, G_GRASS) {
                continue;
            }
            let grove = value_noise(cx as f32 / 14.0, cy as f32 / 14.0, salt);
            let roll = (hash(cx, cy, salt + 13) % 1000) as f32 / 1000.0;
            if roll > chance * (0.35 + grove * 0.9) {
                continue;
            }
            let variant = hash(cx, cy, salt + 29) as usize % frames.len();
            let frame = frames[variant];
            let vw = (frame.w + CELL_SIZE - 1) / CELL_SIZE;
            let vh = (frame.h + CELL_SIZE - 1) / CELL_SIZE;
            // One occupant per 16×16 cell: the visual footprint cannot sit on a taken cell.
            // Tree occupancy stays ow×oh (trunk); overhang cells can still hold floor plants.
            if rect_has_kind(&layers.ground, cx, cy, vw, vh, G_TAKEN) {
                continue;
            }
            let px = (cx * CELL_SIZE) as f32;
            let py = (cy * CELL_SIZE) as f32;
            if let (Some(shadow), Some((blobs, casters))) = (shadow, shadow_pass.as_mut()) {
                if height > 0.5 {
                    push_shadow(
                        blobs,
                        casters,
                        shadow,
                        px + frame.w as f32 * 0.5,
                        py + frame.h as f32 * 0.86,
                        height,
                    );
                }
            }
            let mut inst = instance_from_frame(px.round(), py.round(), frame);
            inst.flags = wind;
            sprites.push(inst);
            if let Some(rank) = rank.as_mut() {
                rank.push((height, py + frame.h as f32));
            }
            mark_rect(&mut layers.ground, cx, cy, ow, oh, G_TAKEN);
            if covers {
                // Trees: only the trunk occupancy cells. Canopy is visual, not cover.
                if height >= 28.0 {
                    let tw = 1u32.max(ow / 2);
                    let tcx = cx + vw.saturating_sub(tw) / 2;
                    let tcy = cy + vh.saturating_sub(1);
                    layers.mark_cover(tcx, tcy, tw, 1);
                    layers.mark_solid(tcx, tcy, tw, 1);
                } else {
                    layers.mark_cover(cx, cy, ow, oh);
                }
            } else if floor_kind != 0 {
                layers.mark_floor(cx, cy, ow, oh, floor_kind);
            }
        }
    }
    Ok(())
}

fn rect_is(grid: &[u8], cx: u32, cy: u32, ow: u32, oh: u32, kind: u8) -> bool {
    let x1 = (cx + ow).min(GRID_WIDTH);
    let y1 = (cy + oh).min(GRID_HEIGHT);
    if x1.saturating_sub(cx) < ow || y1.saturating_sub(cy) < oh {
        return false;
    }
    for y in cy..y1 {
        for x in cx..x1 {
            if grid[(y * GRID_WIDTH + x) as usize] != kind {
                return false;
            }
        }
    }
    true
}

fn rect_has_kind(grid: &[u8], cx: u32, cy: u32, ow: u32, oh: u32, kind: u8) -> bool {
    let x1 = (cx + ow).min(GRID_WIDTH);
    let y1 = (cy + oh).min(GRID_HEIGHT);
    for y in cy..y1 {
        for x in cx..x1 {
            if grid[(y * GRID_WIDTH + x) as usize] == kind {
                return true;
            }
        }
    }
    false
}

fn mark_rect(grid: &mut [u8], cx: u32, cy: u32, ow: u32, oh: u32, kind: u8) {
    for y in cy..cy + oh {
        for x in cx..cx + ow {
            grid[(y * GRID_WIDTH + x) as usize] = kind;
        }
    }
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

fn push_shadow(
    instances: &mut Vec<Instance>,
    casters: &mut Vec<ShadowCaster>,
    shadow: &Frame,
    origin_x: f32,
    origin_y: f32,
    height: f32,
) {
    let index = instances.len() as u32;
    instances.push(instance_from_frame(
        origin_x - shadow.w as f32 * 0.5,
        origin_y - shadow.h as f32 * 0.45,
        shadow,
    ));
    casters.push(ShadowCaster {
        shadow_index: index,
        origin: [origin_x, origin_y],
        height,
        base_size: [shadow.w as f32, shadow.h as f32],
    });
}

pub fn update_dynamic_shadows(
    instances: &mut [Instance],
    casters: &[ShadowCaster],
    sun_dir: glam::Vec2,
    elevation: f32,
    shadow_str: f32,
) {
    let mut dir = if sun_dir.length_squared() < 0.0001 {
        glam::Vec2::new(0.72, 0.70)
    } else {
        sun_dir.normalize()
    };
    dir.x = dir.x.abs().max(0.35);
    dir.y = dir.y.abs().max(0.50);
    dir = dir.normalize();
    // Overhead (elevation ~1) → circular. Side/low sun → long oval.
    let low = (1.0 - elevation.clamp(0.25, 1.0)).clamp(0.0, 1.0);
    for caster in casters {
        let Some(instance) = instances.get_mut(caster.shadow_index as usize) else {
            continue;
        };
        let plant = (caster.height / 20.0).clamp(0.8, 1.45);
        let size = (caster.base_size[0] + caster.base_size[1]) * 0.5 * plant;
        let w = (size * (1.0 + low * 0.95)).round().max(8.0);
        let h = (size * (1.0 - low * 0.52)).round().max(6.0);
        let offset = dir * (1.2 + caster.height * (0.05 + low * 0.30));
        instance.world_xy = [
            (caster.origin[0] + offset.x - w * (0.32 + low * 0.18)).round(),
            (caster.origin[1] + offset.y - h * (0.50 + low * 0.08)).round(),
        ];
        instance.size_xy = [w, h];
        instance.alpha = shadow_str.clamp(0.18, 1.0);
    }
}

pub fn update_cover_fade(cover: &mut [Instance], probe: glam::Vec2) {
    const GHOST: f32 = 0.32;
    let mut best: Option<(usize, f32)> = None;
    for (i, instance) in cover.iter().enumerate() {
        let cx = instance.world_xy[0] + instance.size_xy[0] * 0.5;
        let cy = instance.world_xy[1] + instance.size_xy[1] * 0.42;
        let rx = instance.size_xy[0] * 0.38;
        let ry = instance.size_xy[1] * 0.38;
        let nx = (probe.x - cx) / rx.max(1.0);
        let ny = (probe.y - cy) / ry.max(1.0);
        let d = nx * nx + ny * ny;
        if d <= 1.0 {
            let score = d * 40.0 + instance.size_xy[1];
            if best.map(|(_, s)| score < s).unwrap_or(true) {
                best = Some((i, score));
            }
        }
    }
    let ghost_i = best.map(|(i, _)| i);
    for (i, instance) in cover.iter_mut().enumerate() {
        let target = if Some(i) == ghost_i { GHOST } else { 1.0 };
        instance.alpha += (target - instance.alpha) * 0.22;
    }
}

pub fn instance_from_frame(world_x: f32, world_y: f32, frame: &Frame) -> Instance {
    Instance {
        world_xy: [world_x.round(), world_y.round()],
        atlas_xy: [frame.x as f32, frame.y as f32],
        size_xy: [frame.w as f32, frame.h as f32],
        atlas_wh: [frame.w as f32, frame.h as f32],
        alpha: 1.0,
        flags: 0.0,
    }
}

fn hash(x: u32, y: u32, salt: u32) -> u32 {
    let mut n = x.wrapping_mul(374761393) ^ y.wrapping_mul(668265263) ^ salt;
    n = (n ^ (n >> 13)).wrapping_mul(1274126177);
    n ^ (n >> 16)
}
