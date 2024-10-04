use crate::tilemap::{TileMap, TILE_SIZE};
use notan::math::Vec2;

pub struct Player {
    pub pos: Vec2,
    pub vel: Vec2,
    pub size: Vec2,
    pub on_ground: bool,
}

impl Player {
    pub fn new(x: f32, y: f32) -> Self {
        Player {
            pos: Vec2::new(x, y),
            vel: Vec2::ZERO,
            size: Vec2::new(8.0, 8.0),
            on_ground: false,
        }
    }

    pub fn update(&mut self, tilemap: &TileMap, dt: f32) {
        // Apply gravity
        if !self.on_ground {
            self.vel.y += 800.0 * dt; // Gravity constant
            self.vel.y = self.vel.y.min(400.0); // Terminal velocity
        }

        // Horizontal movement and collision
        self.handle_horizontal_movement(tilemap, dt);

        // Vertical movement and collision
        self.handle_vertical_movement(tilemap, dt);

        // Check if on ground
        self.check_on_ground(tilemap);
    }

    fn handle_horizontal_movement(&mut self, tilemap: &TileMap, dt: f32) {
        let move_amount = self.vel.x * dt;
        let mut remaining_movement = move_amount.abs();
        let direction = move_amount.signum();

        while remaining_movement > 0.0 {
            let step = remaining_movement.min(1.0) * direction;
            let new_x = self.pos.x + step;

            if self.can_move_to(tilemap, new_x, self.pos.y) {
                self.pos.x = new_x;
                self.try_step_down(tilemap);
                remaining_movement -= step.abs();
            } else if self.can_move_to(tilemap, new_x, self.pos.y - 1.0) {
                // Step up
                self.pos.x = new_x;
                self.pos.y -= 1.0;
                remaining_movement -= step.abs();
            } else {
                // Collision, stop movement
                self.vel.x = 0.0;
                break;
            }
        }
    }

    fn can_move_to(&self, tilemap: &TileMap, x: f32, y: f32) -> bool {
        let left = x;
        let right = x + self.size.x - 1.0;
        let top = y;
        let bottom = y + self.size.y - 1.0;

        !tilemap.is_pixel_solid(left, top)
            && !tilemap.is_pixel_solid(right, top)
            && !tilemap.is_pixel_solid(left, bottom)
            && !tilemap.is_pixel_solid(right, bottom)
    }

    fn try_step_down(&mut self, tilemap: &TileMap) {
        let max_step_down = 3; // Maximum pixels to step down
        for step in 1..=max_step_down {
            let test_y = self.pos.y + step as f32;
            if self.can_move_to(tilemap, self.pos.x, test_y) {
                if tilemap.is_pixel_solid(self.pos.x, test_y + self.size.y)
                    || tilemap.is_pixel_solid(self.pos.x + self.size.x - 1.0, test_y + self.size.y)
                {
                    self.pos.y = test_y;
                    break;
                }
            } else {
                break;
            }
        }
    }

    fn handle_vertical_movement(&mut self, tilemap: &TileMap, dt: f32) {
        let move_amount = self.vel.y * dt;
        let mut new_y = self.pos.y + move_amount;

        if move_amount > 0.0 {
            // Moving down
            let floor_y = self.find_floor(tilemap, new_y);
            if floor_y < new_y + self.size.y {
                new_y = floor_y - self.size.y;
                self.vel.y = 0.0;
                self.on_ground = true;
            }
        } else {
            // Moving up
            if self.check_ceiling(tilemap, new_y) {
                new_y = new_y.ceil();
                self.vel.y = 0.0;
            }
        }

        self.pos.y = new_y;
    }

    fn find_floor(&self, tilemap: &TileMap, start_y: f32) -> f32 {
        let mut y = start_y;
        let bottom = start_y + self.size.y;
        while y <= bottom && y < (tilemap.height as f32 * TILE_SIZE) {
            if tilemap.is_pixel_solid(self.pos.x, y)
                || tilemap.is_pixel_solid(self.pos.x + self.size.x - 1.0, y)
            {
                return y;
            }
            y += 1.0;
        }
        bottom
    }

    fn check_ceiling(&self, tilemap: &TileMap, y: f32) -> bool {
        tilemap.is_pixel_solid(self.pos.x, y)
            || tilemap.is_pixel_solid(self.pos.x + self.size.x - 1.0, y)
    }

    fn check_on_ground(&mut self, tilemap: &TileMap) {
        let feet_y = self.pos.y + self.size.y;
        self.on_ground = tilemap.is_pixel_solid(self.pos.x, feet_y)
            || tilemap.is_pixel_solid(self.pos.x + self.size.x - 1.0, feet_y);
    }

    pub fn move_horizontal(&mut self, left: bool, right: bool, dt: f32) {
        const ACCELERATION: f32 = 800.0;
        const MAX_SPEED: f32 = 96.0;

        if left {
            self.vel.x -= ACCELERATION * dt;
        } else if right {
            self.vel.x += ACCELERATION * dt;
        } else {
            // Apply friction
            let friction = 1000.0 * dt;
            if self.vel.x > 0.0 {
                self.vel.x = (self.vel.x - friction).max(0.0);
            } else if self.vel.x < 0.0 {
                self.vel.x = (self.vel.x + friction).min(0.0);
            }
        }

        // Clamp horizontal speed
        self.vel.x = self.vel.x.clamp(-MAX_SPEED, MAX_SPEED);
    }

    pub fn jump(&mut self) {
        if self.on_ground {
            self.vel.y = -325.0; // Jump force
            self.on_ground = false;
        }
    }
}
