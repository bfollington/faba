use crate::tilemap::{SlopeType, TileMap, TileType, TILE_SIZE};
use bitflags::bitflags;
use macroquad::prelude::*;

bitflags! {
    pub struct CollisionFlags: u16 {
        const NONE = 0;
        const LEFT_WALL = 1 << 0;
        const RIGHT_WALL = 1 << 1;
        const TOP_WALL = 1 << 2;
        const BOTTOM_WALL = 1 << 3;
        const LEFT_SLOPE = 1 << 4;
        const RIGHT_SLOPE = 1 << 5;
        const IN_WATER = 1 << 6;
    }
}

#[derive(Clone, Copy, Debug)]
enum CollisionType {
    Solid,
    Slope(SlopeType),
}

pub struct Player {
    pub pos: Vec2,
    pub vel: Vec2,
    pub size: Vec2,
    pub collision_flags: CollisionFlags,
    pub direction: Direction,
    jump_buffer: f32,
    coyote_time: f32,
    is_jumping: bool,
    pub last_collision: Option<CollisionType>,
}

#[derive(PartialEq, Clone, Copy)]
pub enum Direction {
    Left,
    Right,
}

const MOVE_SPEED: f32 = 15.;
const JUMP_FORCE: f32 = -3.5;
const GRAVITY: f32 = 98.;
const JUMP_BUFFER_TIME: f32 = 0.1;
const COYOTE_TIME: f32 = 0.1;
const GROUND_FRICTION: f32 = 0.85;
const AIR_RESISTANCE: f32 = 0.95;
const MAX_FALL_SPEED: f32 = 60.0;
const GROUND_TOLERANCE: f32 = 0.1; // Small tolerance for ground collision

impl Player {
    pub fn new(x: f32, y: f32) -> Self {
        Player {
            pos: vec2(x, y),
            vel: vec2(0.0, 0.0),
            size: vec2(10.0, 20.0),
            collision_flags: CollisionFlags::NONE,
            direction: Direction::Right,
            jump_buffer: 0.0,
            coyote_time: 0.0,
            is_jumping: false,
            last_collision: None,
        }
    }

    pub fn set_movement(&mut self, move_left: bool, move_right: bool) {
        let acceleration = if self.is_grounded() {
            MOVE_SPEED
        } else {
            MOVE_SPEED * 0.75
        };

        if move_left {
            self.vel.x -= acceleration;
            self.direction = Direction::Left;
        } else if move_right {
            self.vel.x += acceleration;
            self.direction = Direction::Right;
        } else if self.is_grounded() {
            self.vel.x *= GROUND_FRICTION;
        }

        // Apply air resistance
        if !self.is_grounded() {
            self.vel.x *= AIR_RESISTANCE;
        }

        // Clamp horizontal velocity
        self.vel.x = self.vel.x.clamp(-MOVE_SPEED * 2.0, MOVE_SPEED * 2.0);
    }

    pub fn jump(&mut self) {
        if self.is_grounded() || self.coyote_time > 0.0 {
            self.vel.y = JUMP_FORCE;
            self.is_jumping = true;
            self.coyote_time = 0.0;
        } else {
            self.jump_buffer = JUMP_BUFFER_TIME;
        }
    }

    pub fn release_jump(&mut self) {
        if self.is_jumping && self.vel.y < 0.0 {
            self.vel.y *= 0.5;
        }
        self.is_jumping = false;
    }

    fn get_collision_bounds(&self) -> Rect {
        Rect::new(self.pos.x, self.pos.y, self.size.x, self.size.y)
    }

    pub fn is_grounded(&self) -> bool {
        self.collision_flags.intersects(
            CollisionFlags::BOTTOM_WALL | CollisionFlags::LEFT_SLOPE | CollisionFlags::RIGHT_SLOPE,
        )
    }

    fn move_vertical(&mut self, tilemap: &TileMap, dt: f32) {
        let move_amount = self.vel.y * dt;
        let new_y = self.pos.y + move_amount;

        let (start_y, end_y) = if move_amount > 0.0 {
            (self.pos.y, new_y + self.size.y)
        } else {
            (new_y, self.pos.y + self.size.y)
        };

        let start_tile_x = (self.pos.x / TILE_SIZE).floor() as i32;
        let end_tile_x = ((self.pos.x + self.size.x - 1.0) / TILE_SIZE).floor() as i32;
        let start_tile_y = (start_y / TILE_SIZE).floor() as i32;
        let end_tile_y = (end_y / TILE_SIZE).ceil() as i32;

        self.collision_flags.remove(
            CollisionFlags::BOTTOM_WALL | CollisionFlags::LEFT_SLOPE | CollisionFlags::RIGHT_SLOPE,
        );

        for tile_y in start_tile_y..=end_tile_y {
            for tile_x in start_tile_x..=end_tile_x {
                if let Some(collision) = self.check_collision(tilemap, tile_x, tile_y) {
                    match collision {
                        CollisionType::Solid => {
                            if move_amount > 0.0 {
                                self.pos.y =
                                    tile_y as f32 * TILE_SIZE - self.size.y - GROUND_TOLERANCE;
                                self.collision_flags.insert(CollisionFlags::BOTTOM_WALL);
                            } else {
                                self.pos.y = (tile_y as f32 + 1.0) * TILE_SIZE;
                                self.collision_flags.insert(CollisionFlags::TOP_WALL);
                            }
                            self.vel.y = 0.0;
                            return;
                        }
                        CollisionType::Slope(slope_type) => {
                            if self.adjust_to_slope(tile_x, tile_y, slope_type) {
                                return;
                            }
                        }
                    }
                }
            }
        }

        self.pos.y = new_y;
    }

    fn adjust_to_slope(&mut self, tile_x: i32, tile_y: i32, slope_type: SlopeType) -> bool {
        let tile_pos = Vec2::new(tile_x as f32 * TILE_SIZE, tile_y as f32 * TILE_SIZE);
        let player_bottom = self.pos.y + self.size.y;
        let slope_y = match slope_type {
            SlopeType::RightUp => tile_pos.y + TILE_SIZE - (self.pos.x + self.size.x - tile_pos.x),
            SlopeType::LeftUp => tile_pos.y + (self.pos.x - tile_pos.x),
        };

        if player_bottom > slope_y {
            self.pos.y = slope_y - self.size.y;
            self.vel.y = 0.0;
            self.collision_flags.insert(match slope_type {
                SlopeType::RightUp => CollisionFlags::RIGHT_SLOPE,
                SlopeType::LeftUp => CollisionFlags::LEFT_SLOPE,
            });
            true
        } else {
            false
        }
    }

    fn move_horizontal(&mut self, tilemap: &TileMap, dt: f32) {
        let move_amount = self.vel.x * dt;
        let new_x = self.pos.x + move_amount;

        let (start_x, end_x) = if move_amount > 0.0 {
            (self.pos.x, new_x + self.size.x)
        } else {
            (new_x, self.pos.x + self.size.x)
        };

        let start_tile_x = (start_x / TILE_SIZE).floor() as i32;
        let end_tile_x = (end_x / TILE_SIZE).ceil() as i32;
        let start_tile_y = (self.pos.y / TILE_SIZE).floor() as i32;
        let end_tile_y = ((self.pos.y + self.size.y - 1.0) / TILE_SIZE).floor() as i32;

        self.collision_flags
            .remove(CollisionFlags::LEFT_WALL | CollisionFlags::RIGHT_WALL);

        for tile_x in start_tile_x..=end_tile_x {
            for tile_y in start_tile_y..=end_tile_y {
                if let Some(collision) = self.check_collision(tilemap, tile_x, tile_y) {
                    match collision {
                        CollisionType::Solid => {
                            if move_amount > 0.0 {
                                self.pos.x = tile_x as f32 * TILE_SIZE - self.size.x;
                                self.collision_flags.insert(CollisionFlags::RIGHT_WALL);
                            } else if move_amount < 0.0 {
                                self.pos.x = (tile_x as f32 + 1.0) * TILE_SIZE;
                                self.collision_flags.insert(CollisionFlags::LEFT_WALL);
                            }
                            self.vel.x = 0.0;
                            return;
                        }
                        CollisionType::Slope(_) => {
                            // Slopes are handled in vertical movement
                        }
                    }
                }
            }
        }

        // Only update position if no collision occurred
        self.pos.x = new_x;
    }

    pub fn update(&mut self, tilemap: &TileMap, dt: f32) {
        if !self.is_grounded() {
            self.apply_gravity(dt);
        } else {
            self.vel.y = 0.0;
        }

        // Apply friction before movement
        if self.is_grounded() {
            self.vel.x *= 0.9; // Adjust friction coefficient as needed
        }

        self.move_horizontal(tilemap, dt);
        self.move_vertical(tilemap, dt);

        // Ensure velocity is zeroed if very small to prevent drift
        if self.vel.x.abs() < 0.01 {
            self.vel.x = 0.0;
        }
        if self.vel.y.abs() < 0.01 {
            self.vel.y = 0.0;
        }
    }

    fn check_collision(&self, tilemap: &TileMap, x: i32, y: i32) -> Option<CollisionType> {
        if x < 0 || y < 0 || x >= tilemap.width as i32 || y >= tilemap.height as i32 {
            return None;
        }

        let tile_world_pos = Vec2::new(x as f32 * TILE_SIZE, y as f32 * TILE_SIZE);
        let tile_rect = Rect::new(tile_world_pos.x, tile_world_pos.y, TILE_SIZE, TILE_SIZE);

        if self.get_collision_bounds().overlaps(&tile_rect) {
            match tilemap.get_tile(x as usize, y as usize) {
                TileType::Solid => Some(CollisionType::Solid),
                TileType::Slope(slope_type) => Some(CollisionType::Slope(slope_type)),
                _ => None,
            }
        } else {
            None
        }
    }

    fn apply_gravity(&mut self, dt: f32) {
        self.vel.y += GRAVITY * dt;
        self.vel.y = self.vel.y.min(MAX_FALL_SPEED);
    }
}

impl Player {
    pub fn debug_render(&self, tilemap: &TileMap) {
        // Player collision bounds
        let bounds = self.get_collision_bounds();
        draw_rectangle_lines(bounds.x, bounds.y, bounds.w, bounds.h, 2.0, GREEN);

        // Checked tiles
        let (start_tile_x, start_tile_y) = (
            (bounds.x / TILE_SIZE).floor() as i32,
            (bounds.y / TILE_SIZE).floor() as i32,
        );
        let (end_tile_x, end_tile_y) = (
            ((bounds.x + bounds.w) / TILE_SIZE).ceil() as i32,
            ((bounds.y + bounds.h) / TILE_SIZE).ceil() as i32,
        );

        for tile_y in start_tile_y..end_tile_y {
            for tile_x in start_tile_x..end_tile_x {
                draw_rectangle_lines(
                    tile_x as f32 * TILE_SIZE,
                    tile_y as f32 * TILE_SIZE,
                    TILE_SIZE,
                    TILE_SIZE,
                    1.0,
                    RED,
                );
            }
        }

        // Collision flags
        let mut flag_text = String::new();
        if self.collision_flags.contains(CollisionFlags::LEFT_WALL) {
            flag_text += "L ";
        }
        if self.collision_flags.contains(CollisionFlags::RIGHT_WALL) {
            flag_text += "R ";
        }
        if self.collision_flags.contains(CollisionFlags::TOP_WALL) {
            flag_text += "T ";
        }
        if self.collision_flags.contains(CollisionFlags::BOTTOM_WALL) {
            flag_text += "B ";
        }
        if self.collision_flags.contains(CollisionFlags::LEFT_SLOPE) {
            flag_text += "SL ";
        }
        if self.collision_flags.contains(CollisionFlags::RIGHT_SLOPE) {
            flag_text += "SR ";
        }
        draw_text(&flag_text, self.pos.x, self.pos.y - 20.0, 20.0, WHITE);

        // Velocity
        draw_line(
            self.pos.x + self.size.x / 2.0,
            self.pos.y + self.size.y / 2.0,
            self.pos.x + self.size.x / 2.0 + self.vel.x * 10.0,
            self.pos.y + self.size.y / 2.0 + self.vel.y * 10.0,
            2.0,
            BLUE,
        );

        // Enhanced debug info
        let debug_info = format!(
                    "Pos: ({:.2}, {:.2})\nVel: ({:.2}, {:.2})\nGrounded: {}\nCollision Flags: {:?}\nLast Collision: {:?}",
                    self.pos.x, self.pos.y,
                    self.vel.x, self.vel.y,
                    self.is_grounded(),
                    self.collision_flags,
                    self.last_collision
                );
        draw_text(&debug_info, 10.0, 80.0, 20.0, WHITE);

        // Horizontal movement visualization
        let move_end = self.pos + self.vel;
        draw_line(
            self.pos.x + self.size.x / 2.0,
            self.pos.y + self.size.y / 2.0,
            move_end.x + self.size.x / 2.0,
            move_end.y + self.size.y / 2.0,
            2.0,
            YELLOW,
        );

        // Collision points
        for tile_y in
            (self.pos.y / TILE_SIZE) as i32..=((self.pos.y + self.size.y) / TILE_SIZE) as i32
        {
            for tile_x in
                (self.pos.x / TILE_SIZE) as i32..=((self.pos.x + self.size.x) / TILE_SIZE) as i32
            {
                if let Some(collision) = self.check_collision(tilemap, tile_x, tile_y) {
                    let color = match collision {
                        CollisionType::Solid => RED,
                        CollisionType::Slope(_) => GREEN,
                    };
                    draw_circle(
                        tile_x as f32 * TILE_SIZE,
                        tile_y as f32 * TILE_SIZE,
                        3.0,
                        color,
                    );
                }
            }
        }
    }
}
