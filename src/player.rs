use glam::Vec2;

const SPEED: f32 = 86.0;
pub const VIS: f32 = 32.0;
const HIT_W: f32 = 10.0;
const HIT_H: f32 = 8.0;
const HIT_OX: f32 = (VIS - HIT_W) * 0.5;
const HIT_OY: f32 = VIS - HIT_H - 1.0;

const FRAME_IDS: [[&str; 2]; 4] = [
    ["player_e", "player_e2"],
    ["player_n", "player_n2"],
    ["player_w", "player_w2"],
    ["player_s", "player_s2"],
];

pub struct Player {
    pub pos: Vec2,
    facing: usize,
    walk_t: f32,
    moving: bool,
}

impl Player {
    pub fn new(pos: Vec2) -> Self {
        Self {
            pos,
            facing: 3,
            walk_t: 0.0,
            moving: false,
        }
    }

    pub fn center(&self) -> Vec2 {
        self.pos + Vec2::splat(VIS * 0.5)
    }

    pub fn foot_y(&self) -> f32 {
        self.pos.y + VIS
    }

    pub fn frame_id(&self) -> &'static str {
        let step = if self.moving && self.walk_t as u32 % 2 == 1 {
            1
        } else {
            0
        };
        FRAME_IDS[self.facing][step]
    }

    pub fn tick(
        &mut self,
        axis: Vec2,
        dt: f32,
        blocked: impl Fn(f32, f32, f32, f32) -> bool,
    ) {
        if axis.length_squared() < 0.0001 {
            self.moving = false;
            self.walk_t = 0.0;
            return;
        }
        self.moving = true;
        self.walk_t += dt * 8.0;
        if axis.x.abs() >= axis.y.abs() {
            self.facing = if axis.x > 0.0 { 0 } else { 2 };
        } else {
            self.facing = if axis.y < 0.0 { 1 } else { 3 };
        }
        let vel = axis * SPEED * dt;
        let (hx, hy, hw, hh) = self.hitbox();
        if !blocked(hx + vel.x, hy, hw, hh) {
            self.pos.x += vel.x;
        }
        let (hx, hy, hw, hh) = self.hitbox();
        if !blocked(hx, hy + vel.y, hw, hh) {
            self.pos.y += vel.y;
        }
    }

    fn hitbox(&self) -> (f32, f32, f32, f32) {
        (self.pos.x + HIT_OX, self.pos.y + HIT_OY, HIT_W, HIT_H)
    }
}
