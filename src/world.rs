use bytemuck::{Pod, Zeroable};

pub const TILE_SIZE: u32 = 32;
pub const LOGICAL_WIDTH: f32 = 640.0;
pub const LOGICAL_HEIGHT: f32 = 360.0;
pub const MAP_WIDTH: u32 = 64;
pub const MAP_HEIGHT: u32 = 40;
pub const ATLAS_TILES: u32 = 4;

pub const TILE_GRASS_A: u32 = 0;
pub const TILE_GRASS_B: u32 = 1;
pub const TILE_DIRT: u32 = 2;
pub const TILE_STONE: u32 = 3;

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

pub fn tile_at(x: u32, y: u32) -> u32 {
    let path_center = 18.0 + (x as f32 * 0.18).sin() * 5.0;
    let dy = (y as f32 - path_center).abs();
    if dy < 1.6 {
        TILE_STONE
    } else if dy < 2.6 {
        TILE_DIRT
    } else if (x + y * 3) % 11 == 0 {
        TILE_GRASS_B
    } else {
        TILE_GRASS_A
    }
}

pub fn instances() -> Vec<Instance> {
    let mut out = Vec::with_capacity((MAP_WIDTH * MAP_HEIGHT) as usize);
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let tile = tile_at(x, y);
            out.push(Instance {
                world_xy: [(x * TILE_SIZE) as f32, (y * TILE_SIZE) as f32],
                atlas_xy: [(tile * TILE_SIZE) as f32, 0.0],
            });
        }
    }
    out
}

pub fn atlas_rgba() -> (Vec<u8>, u32, u32) {
    let width = TILE_SIZE * ATLAS_TILES;
    let height = TILE_SIZE;
    let mut pixels = vec![0u8; (width * height * 4) as usize];
    paint_tile(&mut pixels, width, 0, grass_a);
    paint_tile(&mut pixels, width, 1, grass_b);
    paint_tile(&mut pixels, width, 2, dirt);
    paint_tile(&mut pixels, width, 3, stone);
    (pixels, width, height)
}

fn paint_tile(pixels: &mut [u8], atlas_width: u32, tile: u32, shade: fn(u32, u32) -> [u8; 4]) {
    let ox = tile * TILE_SIZE;
    for y in 0..TILE_SIZE {
        for x in 0..TILE_SIZE {
            let color = shade(x, y);
            let i = (((y * atlas_width) + ox + x) * 4) as usize;
            pixels[i..i + 4].copy_from_slice(&color);
        }
    }
}

fn hash(x: u32, y: u32, salt: u32) -> u32 {
    let mut n = x.wrapping_mul(374761393) ^ y.wrapping_mul(668265263) ^ salt;
    n = (n ^ (n >> 13)).wrapping_mul(1274126177);
    n ^ (n >> 16)
}

fn grass_a(x: u32, y: u32) -> [u8; 4] {
    let n = hash(x, y, 11) % 100;
    let blade = hash(x, y, 3) % 17 == 0;
    if blade {
        [62, 92, 48, 255]
    } else if n < 20 {
        [38, 58, 36, 255]
    } else if n < 55 {
        [46, 70, 40, 255]
    } else {
        [52, 78, 44, 255]
    }
}

fn grass_b(x: u32, y: u32) -> [u8; 4] {
    let n = hash(x, y, 29) % 100;
    let speck = hash(x, y, 7) % 23 == 0;
    if speck {
        [86, 98, 42, 255]
    } else if n < 30 {
        [34, 54, 32, 255]
    } else {
        [42, 64, 38, 255]
    }
}

fn dirt(x: u32, y: u32) -> [u8; 4] {
    let n = hash(x, y, 41) % 100;
    let pebble = hash(x, y, 19) % 31 == 0;
    if pebble {
        [92, 82, 68, 255]
    } else if n < 25 {
        [72, 54, 38, 255]
    } else if n < 60 {
        [84, 62, 42, 255]
    } else {
        [96, 72, 48, 255]
    }
}

fn stone(x: u32, y: u32) -> [u8; 4] {
    let cell_x = x / 8;
    let cell_y = y / 8;
    let local_x = x % 8;
    let local_y = y % 8;
    let grout = local_x == 0 || local_y == 0;
    if grout {
        return [58, 52, 48, 255];
    }
    let n = hash(cell_x, cell_y, 53) % 4;
    let highlight = local_x == 2 && local_y == 2;
    let base = match n {
        0 => [118, 112, 104, 255],
        1 => [108, 104, 98, 255],
        2 => [124, 116, 106, 255],
        _ => [112, 108, 100, 255],
    };
    if highlight {
        [148, 140, 126, 255]
    } else {
        base
    }
}
