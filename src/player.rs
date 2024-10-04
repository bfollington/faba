use crate::tilemap::{TileMap, TileType, TILE_SIZE};
use notan::draw::*;
use notan::math::Vec2;
use notan::prelude::*;

pub struct Player {
    pub pos: Vec2,
    pub velocity: Vec2,
    pub temp_velocity: Vec2,
    pub size: Vec2,
    pub on_ground: bool,
    pub acceleration: Vec2,
    pub friction: Vec2,
    pub max_speed: Vec2,
    pub moved_amount: Vec2,
    collision_types: Vec<TileType>,
    jump_timer: f32,
    max_jump_time: f32,
    jump_force: f32,
    sprint_speed_multiplier: f32,
}

impl Player {
    pub fn new(x: f32, y: f32) -> Self {
        Player {
            pos: Vec2::new(x, y),
            velocity: Vec2::ZERO,
            temp_velocity: Vec2::ZERO,
            size: Vec2::new(8.0, 8.0),
            on_ground: false,
            acceleration: Vec2::new(0.0, 0.01),
            friction: Vec2::new(1.15, 1.0),
            max_speed: Vec2::new(2., 10.0),
            moved_amount: Vec2::ZERO,
            collision_types: vec![
                TileType::Solid,
                TileType::SlopeUpRight,
                TileType::SlopeUpLeft,
            ],
            jump_timer: 0.0,
            max_jump_time: 0.1, // Maximum time the jump button can be held for higher jumps
            jump_force: -5.0,   // Initial jump force
            sprint_speed_multiplier: 2., // Speed multiplier when sprinting
        }
    }

    pub fn update(&mut self, tilemap: &TileMap, dt: f32, jump_button_held: bool) {
        self.collision_bottom(tilemap);

        // Handle variable height jumping
        if jump_button_held && self.jump_timer > 0.0 {
            self.velocity.y = self.jump_force;
            self.jump_timer -= dt;
            if self.jump_timer <= 0.0 {
                self.jump_timer = 0.0;
            }
        } else {
            self.jump_timer = 0.0;
        }

        if !self.on_ground {
            self.velocity.y += self.acceleration.y;
        }

        self.velocity.x /= self.friction.x;
        if !self.on_ground {
            self.velocity.y /= self.friction.y;
        }

        self.move_object(tilemap, dt);
        self.clamp_speed();
    }

    fn move_object(&mut self, tilemap: &TileMap, dt: f32) {
        self.temp_velocity.x += self.velocity.x.abs() * dt * 60.0;
        self.temp_velocity.y += self.velocity.y.abs() * dt * 60.0;

        let horizontal_direction = self.velocity.x.signum() as i32;
        let vertical_direction = self.velocity.y.signum() as i32;

        while self.temp_velocity.y >= 1.0 {
            self.move_y(tilemap, vertical_direction);
            self.temp_velocity.y -= 1.0;
        }

        while self.temp_velocity.x >= 1.0 {
            if self.move_x(tilemap, horizontal_direction) {
                self.temp_velocity.x -= 1.0;
            } else {
                break;
            }
        }
    }

    fn move_x(&mut self, tilemap: &TileMap, dir: i32) -> bool {
        self.moved_amount.x += 1.0;
        let new_x = self.pos.x + dir as f32;

        if !self.collide(tilemap, new_x, self.pos.y) {
            if !self.collide(tilemap, new_x, self.pos.y + 1.0)
                && self.collide(tilemap, self.pos.x, self.pos.y + 1.0)
            {
                self.pos.y += 1.0;
            }
            self.pos.x = new_x;
            self.try_step_up_slope(tilemap, dir);
        } else if self.try_step_up_slope(tilemap, dir) {
            // Successfully stepped up a slope
        } else {
            self.hit_wall();
            return false;
        }
        true
    }

    fn move_y(&mut self, tilemap: &TileMap, dir: i32) {
        self.moved_amount.y += 1.0;
        let new_y = self.pos.y + dir as f32;

        if !self.collide(tilemap, self.pos.x, new_y) {
            self.pos.y = new_y;
        } else if dir > 0 {
            // Moving down, check for slopes
            self.try_step_down_slope(tilemap);
        }

        if self.collide(tilemap, self.pos.x, self.pos.y - 1.0) {
            self.velocity.y = 0.0;
        }
    }

    fn try_step_up_slope(&mut self, tilemap: &TileMap, dir: i32) -> bool {
        for step in 1..=((TILE_SIZE / 2.0) as i32) {
            if !self.collide(tilemap, self.pos.x + dir as f32, self.pos.y - step as f32) {
                self.pos.x += dir as f32;
                self.pos.y -= step as f32;
                return true;
            }
        }
        false
    }

    fn try_step_down_slope(&mut self, tilemap: &TileMap) {
        let mut step = 1;
        while step <= (TILE_SIZE as i32)
            && !self.collide(tilemap, self.pos.x, self.pos.y + step as f32)
        {
            step += 1;
        }
        if step > 1 {
            self.pos.y += (step - 1) as f32;
        }
    }

    fn collide(&self, tilemap: &TileMap, x: f32, y: f32) -> bool {
        let left = x;
        let right = x + self.size.x - 1.0;
        let top = y;
        let bottom = y + self.size.y - 1.0;

        for check_y in [top, bottom] {
            for check_x in [left, right] {
                let tile_type = tilemap.get_tile_type(check_x, check_y);
                match tile_type {
                    TileType::Solid => return true,
                    TileType::SlopeUpRight => {
                        let tile_x = (check_x / TILE_SIZE).floor() * TILE_SIZE;
                        let tile_y = (check_y / TILE_SIZE).floor() * TILE_SIZE;
                        let slope_y = TILE_SIZE - (check_x - tile_x);
                        if check_y >= tile_y + slope_y {
                            return true;
                        }
                    }
                    TileType::SlopeUpLeft => {
                        let tile_x = (check_x / TILE_SIZE).floor() * TILE_SIZE;
                        let tile_y = (check_y / TILE_SIZE).floor() * TILE_SIZE;
                        let slope_y = check_x - tile_x;
                        if check_y >= tile_y + slope_y {
                            return true;
                        }
                    }
                    _ => {}
                }
            }
        }
        false
    }

    fn collision_bottom(&mut self, tilemap: &TileMap) {
        self.on_ground = false;
        let feet_y = self.pos.y + self.size.y;
        if self.collide(tilemap, self.pos.x, feet_y)
            || self.collide(tilemap, self.pos.x + self.size.x - 1.0, feet_y)
        {
            self.on_ground = true;
        }
    }

    fn hit_wall(&mut self) {
        self.temp_velocity.x = 0.0;
        self.velocity.x = 0.0;
    }

    fn clamp_speed(&mut self) {
        let max_x_speed = if self.velocity.x.abs() > self.max_speed.x {
            self.max_speed.x * self.sprint_speed_multiplier
        } else {
            self.max_speed.x
        };
        self.velocity.x = self.velocity.x.clamp(-max_x_speed, max_x_speed);
        self.velocity.y = self.velocity.y.clamp(-self.max_speed.y, self.max_speed.y);
    }

    pub fn move_horizontal(&mut self, left: bool, right: bool, sprint: bool, dt: f32) {
        let base_acceleration = 420.0;
        let acceleration = if sprint {
            base_acceleration * self.sprint_speed_multiplier
        } else {
            base_acceleration
        };

        if left {
            self.velocity.x -= acceleration * dt;
        } else if right {
            self.velocity.x += acceleration * dt;
        }
    }

    pub fn jump(&mut self) {
        if self.on_ground {
            self.velocity.y = self.jump_force;
            self.jump_timer = self.max_jump_time;
            self.on_ground = false;
        }
    }

    pub fn cancel_jump(&mut self) {
        if self.velocity.y < 0.0 {
            self.velocity.y *= 0.5; // Reduce upward velocity when jump is cancelled
        }
        self.jump_timer = 0.0;
    }

    pub fn render_debug(&self, draw: &mut Draw, tilemap: &TileMap) {
        // Render player bounding box
        draw.rect((self.pos.x, self.pos.y), (self.size.x, self.size.y))
            .stroke_color(Color::RED);

        // Render collision points
        let points = [
            (self.pos.x, self.pos.y),
            (self.pos.x + self.size.x - 1.0, self.pos.y),
            (self.pos.x, self.pos.y + self.size.y - 1.0),
            (
                self.pos.x + self.size.x - 1.0,
                self.pos.y + self.size.y - 1.0,
            ),
        ];

        for point in points.iter() {
            draw.rect((point.0, point.1), (2.0, 2.0))
                .color(Color::GREEN);
        }

        // Draw X velocity vector
        draw.line(
            (
                self.pos.x + self.size.x / 2.0,
                self.pos.y + self.size.y / 2.0,
            ),
            (
                self.pos.x + self.size.x / 2.0 + self.velocity.x * 10.0,
                self.pos.y + self.size.y / 2.0,
            ),
        )
        .color(Color::RED);

        // Draw Y velocity vector
        draw.line(
            (
                self.pos.x + self.size.x / 2.0,
                self.pos.y + self.size.y / 2.0,
            ),
            (
                self.pos.x + self.size.x / 2.0,
                self.pos.y + self.size.y / 2.0 + self.velocity.y * 10.0,
            ),
        )
        .color(Color::BLUE);

        // Render tiles that the player is colliding with
        for y in (self.pos.y as i32 - 1..=(self.pos.y + self.size.y) as i32 + 1) {
            for x in (self.pos.x as i32 - 1..=(self.pos.x + self.size.x) as i32 + 1) {
                let tile_type = tilemap.get_tile_type(x as f32, y as f32);
                match tile_type {
                    TileType::Solid => {
                        draw.rect(
                            (x as f32 * TILE_SIZE, y as f32 * TILE_SIZE),
                            (TILE_SIZE, TILE_SIZE),
                        )
                        .stroke_color(Color::BLUE);
                    }
                    TileType::SlopeUpRight | TileType::SlopeUpLeft => {
                        draw.rect(
                            (x as f32 * TILE_SIZE, y as f32 * TILE_SIZE),
                            (TILE_SIZE, TILE_SIZE),
                        )
                        .stroke_color(Color::YELLOW);

                        // Draw slope line
                        let (start, end) = if tile_type == TileType::SlopeUpRight {
                            (
                                (x as f32 * TILE_SIZE, (y + 1) as f32 * TILE_SIZE),
                                ((x + 1) as f32 * TILE_SIZE, y as f32 * TILE_SIZE),
                            )
                        } else {
                            (
                                (x as f32 * TILE_SIZE, y as f32 * TILE_SIZE),
                                ((x + 1) as f32 * TILE_SIZE, (y + 1) as f32 * TILE_SIZE),
                            )
                        };
                        draw.line((start.0, start.1), (end.0, end.1))
                            .color(Color::YELLOW);
                    }
                    _ => {}
                }
            }
        }
    }
}
