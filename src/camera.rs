use glam::{Mat4, Vec2};

use crate::world::{LOGICAL_HEIGHT, LOGICAL_WIDTH};

const ZONE: Vec2 = Vec2::new(80.0, 48.0);
const FOLLOW_K: f32 = 7.0;

pub struct Camera {
    pub position: Vec2,
    focus: Vec2,
    pub world_size: Vec2,
}

impl Camera {
    pub fn new(world_size: Vec2) -> Self {
        let position = Vec2::new(
            ((world_size.x - LOGICAL_WIDTH).max(0.0) * 0.35).round(),
            ((world_size.y - LOGICAL_HEIGHT).max(0.0) * 0.4).round(),
        );
        Self {
            focus: position
                + Vec2::new(LOGICAL_WIDTH, LOGICAL_HEIGHT) * 0.5,
            position,
            world_size,
        }
    }

    pub fn look_at(&mut self, target: Vec2) {
        self.focus = target;
        self.snap_from_focus();
    }

    /// Deadzone follow, then exponential lerp, then pixel snap.
    /// GDCS camera deadzone + Pixeldex floor-after-lerp.
    pub fn follow(&mut self, target: Vec2, dt: f32) {
        let half = ZONE * 0.5;
        let want = Vec2::new(
            self.focus.x.clamp(target.x - half.x, target.x + half.x),
            self.focus.y.clamp(target.y - half.y, target.y + half.y),
        );
        let a = 1.0 - (-FOLLOW_K * dt).exp();
        self.focus += (want - self.focus) * a;
        self.snap_from_focus();
    }

    fn snap_from_focus(&mut self) {
        let half = Vec2::new(LOGICAL_WIDTH, LOGICAL_HEIGHT) * 0.5;
        let max_x = (self.world_size.x - LOGICAL_WIDTH).max(0.0);
        let max_y = (self.world_size.y - LOGICAL_HEIGHT).max(0.0);
        self.focus.x = self.focus.x.clamp(half.x, half.x + max_x);
        self.focus.y = self.focus.y.clamp(half.y, half.y + max_y);
        let pos = self.focus - half;
        self.position.x = pos.x.clamp(0.0, max_x).floor();
        self.position.y = pos.y.clamp(0.0, max_y).floor();
    }

    pub fn view_proj(&self) -> Mat4 {
        Mat4::orthographic_rh(
            self.position.x,
            self.position.x + LOGICAL_WIDTH,
            self.position.y + LOGICAL_HEIGHT,
            self.position.y,
            -1.0,
            1.0,
        )
    }
}
