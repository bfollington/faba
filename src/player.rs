use crate::tilemap::{TileMap, TILE_SIZE};
use macroquad::prelude::*;

pub struct Player {
    pub pos: Vec2,
    pub vel: Vec2,
    pub size: Vec2,
    pub on_ground: bool,
    pub direction: Direction,
}

enum Direction {
    Left,
    Right,
}

impl Player {
    pub fn new(x: f32, y: f32) -> Self {
        Player {
            pos: vec2(x, y),
            vel: vec2(0.0, 0.0),
            size: vec2(8.0, 8.0),
            on_ground: false,
            direction: Direction::Right,
        }
    }
}

const MAX_SPEED: f32 = 96.0; // pixels per second
const ACCELERATION: f32 = 800.0; // pixels per second squared
const FRICTION: f32 = 1000.0; // pixels per second squared
const GRAVITY: f32 = 800.0; // pixels per second squared
const MAX_FALL_SPEED: f32 = 400.0; // pixels per second
const JUMP_FORCE: f32 = -255.0; // pixels per second

impl Player {
    pub fn move_horizontal(&mut self, left: bool, right: bool, dt: f32) {
        if left {
            self.vel.x -= ACCELERATION * dt;
            self.direction = Direction::Left;
        } else if right {
            self.vel.x += ACCELERATION * dt;
            self.direction = Direction::Right;
        } else {
            // Apply friction
            let friction = FRICTION * dt;
            if self.vel.x > 0.0 {
                self.vel.x = (self.vel.x - friction).max(0.0);
            } else if self.vel.x < 0.0 {
                self.vel.x = (self.vel.x + friction).min(0.0);
            }
        }

        // Clamp horizontal speed
        self.vel.x = self.vel.x.clamp(-MAX_SPEED, MAX_SPEED);

        // Update position
        self.pos.x += self.vel.x * dt;
    }

    pub fn apply_gravity(&mut self, dt: f32) {
        if !self.on_ground {
            self.vel.y += GRAVITY * dt;
            self.vel.y = self.vel.y.min(MAX_FALL_SPEED);
        }

        // Update position
        self.pos.y += self.vel.y * dt;
    }

    pub fn jump(&mut self) {
        if self.on_ground {
            self.vel.y = JUMP_FORCE;
            self.on_ground = false;
        }
    }

    fn collide_with_tiles(&mut self, tilemap: &TileMap, dt: f32) {
        let mut new_pos = self.pos + self.vel * dt;

        // Horizontal collision
        if self.vel.x != 0.0 {
            let check_x = if self.vel.x > 0.0 {
                new_pos.x + self.size.x
            } else {
                new_pos.x
            };
            if tilemap.is_solid(check_x, self.pos.y)
                || tilemap.is_solid(check_x, self.pos.y + self.size.y - 1.0)
            {
                new_pos.x = if self.vel.x > 0.0 {
                    (check_x / TILE_SIZE).floor() * TILE_SIZE - self.size.x
                } else {
                    (check_x / TILE_SIZE).ceil() * TILE_SIZE
                };
                self.vel.x = 0.0;
            }
        }

        // Vertical collision
        self.on_ground = false;
        if self.vel.y != 0.0 {
            let check_y = if self.vel.y > 0.0 {
                new_pos.y + self.size.y
            } else {
                new_pos.y
            };
            if tilemap.is_solid(new_pos.x, check_y)
                || tilemap.is_solid(new_pos.x + self.size.x - 1.0, check_y)
            {
                new_pos.y = if self.vel.y > 0.0 {
                    (check_y / TILE_SIZE).floor() * TILE_SIZE - self.size.y
                } else {
                    (check_y / TILE_SIZE).ceil() * TILE_SIZE
                };
                if self.vel.y > 0.0 {
                    self.on_ground = true;
                }
                self.vel.y = 0.0;
            }
        }

        // Check if on ground when not moving vertically
        if self.vel.y == 0.0 && !self.on_ground {
            let check_y = new_pos.y + self.size.y + 1.0;
            if tilemap.is_solid(new_pos.x, check_y)
                || tilemap.is_solid(new_pos.x + self.size.x - 1.0, check_y)
            {
                self.on_ground = true;
            }
        }

        self.pos = new_pos;
    }

    pub fn update(&mut self, tilemap: &TileMap, dt: f32) {
        self.apply_gravity(dt);
        self.collide_with_tiles(tilemap, dt);
    }
}
