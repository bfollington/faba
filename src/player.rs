use notan::draw::*;
use notan::math::{Rect, Vec2};
use notan::prelude::*;

use sepax2d::aabb::AABB;
use sepax2d::capsule::Capsule;
use sepax2d::circle::Circle;
use sepax2d::polygon::Polygon;
use sepax2d::sat_collision;

use crate::vertices::{ToTuple, ToVec2, Vertices};

const MOVE_SPEED: f32 = 200.0;
const JUMP_FORCE: f32 = -300.0;
const GRAVITY: f32 = 980.0;
const MAX_FALL_SPEED: f32 = 500.0;
const GROUND_FRICTION: f32 = 0.9;
const AIR_RESISTANCE: f32 = 0.99;
const SLOPE_TOLERANCE: f32 = 0.7; // Cosine of maximum slope angle

pub struct Player {
    pub polygon: Polygon,
    pub velocity: Vec2,
    pub size: Vec2,
    pub is_grounded: bool,
    pub jump_buffer: f32,
    pub coyote_time: f32,
    pub last_mtv: Option<(f32, f32)>,
}

impl Player {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        let polygon = Polygon::from_vertices(
            (0., 0.),
            Rect {
                x,
                y,
                width,
                height,
            }
            .vertices(),
        );

        Self {
            polygon,
            velocity: Vec2::ZERO,
            size: Vec2::new(width, height),
            is_grounded: false,
            jump_buffer: 0.0,
            coyote_time: 0.0,
            last_mtv: None,
        }
    }

    pub fn update(&mut self, terrain: &[Polygon], dt: f32) {
        self.apply_gravity(dt);
        self.apply_friction(dt);
        self.move_and_collide(terrain, dt);
        self.update_timers(dt);
    }

    pub fn set_movement(&mut self, move_left: bool, move_right: bool) {
        let mut move_dir = 0.0;
        if move_left {
            move_dir -= 1.0;
        }
        if move_right {
            move_dir += 1.0;
        }

        let target_speed = move_dir * MOVE_SPEED;
        self.velocity.x = self.velocity.x + (target_speed - self.velocity.x) * 0.2;
    }

    pub fn jump(&mut self) {
        if self.is_grounded || self.coyote_time > 0.0 {
            self.velocity.y = JUMP_FORCE;
            self.is_grounded = false;
            self.coyote_time = 0.0;
        } else {
            self.jump_buffer = 0.1; // Set jump buffer for 100ms
        }
    }

    fn apply_gravity(&mut self, dt: f32) {
        self.velocity.y += GRAVITY * dt;
        self.velocity.y = self.velocity.y.min(MAX_FALL_SPEED);
    }

    fn apply_friction(&mut self, dt: f32) {
        if self.is_grounded {
            self.velocity.x *= GROUND_FRICTION.powf(dt * 60.0);
        } else {
            self.velocity.x *= AIR_RESISTANCE.powf(dt * 60.0);
        }
    }

    fn move_and_collide(&mut self, terrain: &[Polygon], dt: f32) {
        let original_position = self.polygon.position.to_vec2();
        let mut new_position = original_position;

        self.is_grounded = false;
        self.last_mtv = None;

        // Apply movement in X axis
        new_position.x += self.velocity.x * dt;
        self.polygon.position = new_position.to_tuple();

        for terrain_poly in terrain {
            let mtv = sat_collision(&self.polygon, terrain_poly);
            if mtv.0 != 0.0 || mtv.1 != 0.0 {
                new_position.x += mtv.0;
                // self.polygon.position = new_position.to_tuple();
                self.last_mtv = Some(mtv);

                // Resolve velocity in X axis
                if (mtv.0 > 0.0 && self.velocity.x < 0.0) || (mtv.0 < 0.0 && self.velocity.x > 0.0)
                {
                    self.velocity.x = 0.0;
                }
            }
        }

        // Apply movement in Y axis
        new_position.y += self.velocity.y * dt;
        self.polygon.position = new_position.to_tuple();

        for terrain_poly in terrain {
            let mtv = sat_collision(&self.polygon, terrain_poly);
            if mtv.0 != 0.0 || mtv.1 != 0.0 {
                new_position.y += mtv.1;
                self.polygon.position = new_position.to_tuple();
                self.last_mtv = Some(mtv);

                // Resolve velocity in Y axis
                if mtv.1 < 0.0 {
                    self.is_grounded = true;
                    if self.velocity.y > 0.0 {
                        self.velocity.y = 0.0;
                    }
                } else if mtv.1 > 0.0 && self.velocity.y < 0.0 {
                    self.velocity.y = 0.0;
                }
            }
        }

        // Update position
        self.polygon.position = new_position.to_tuple();

        // Handle jump buffer
        if self.jump_buffer > 0.0 && self.is_grounded {
            self.velocity.y = JUMP_FORCE;
            self.is_grounded = false;
            self.jump_buffer = 0.0;
        }

        // Debug prints
        println!("Final position: {:?}", new_position);
        println!("Final velocity: {:?}", self.velocity);
    }

    fn update_timers(&mut self, dt: f32) {
        if !self.is_grounded {
            self.coyote_time = (self.coyote_time - dt).max(0.0);
        } else {
            self.coyote_time = 0.1; // Reset coyote time when grounded
        }

        self.jump_buffer = (self.jump_buffer - dt).max(0.0);
    }

    pub fn render(&self, draw: &mut Draw) {
        // Draw player polygon
        let mut path = draw.path();

        path.move_to(
            self.polygon.vertices[0].0 + self.position().x,
            self.polygon.vertices[0].1 + self.position().y,
        );
        for vertex in self.polygon.vertices.iter().skip(1) {
            path.line_to(vertex.0 + self.position().x, vertex.1 + self.position().y);
        }
        path.close();
        path.color(Color::BLUE).fill();
    }

    pub fn position(&self) -> Vec2 {
        self.polygon.position.to_vec2()
    }

    pub fn debug_render(&self, draw: &mut Draw, font: &Font) {
        // Draw velocity vector
        let center = self.polygon.position.to_vec2() + self.size / 2.0;
        let vel_end = center + self.velocity / 5.0; // Scale down for visibility
        draw.line((center.x, center.y), (vel_end.x, vel_end.y))
            .color(Color::RED);

        // Draw grounded indicator
        let grounded_text = if self.is_grounded {
            "Grounded"
        } else {
            "In Air"
        };
        draw.text(font, grounded_text)
            .position(center.x, self.polygon.position.1 - 20.0)
            .size(16.0)
            .color(Color::WHITE);

        // Draw jump buffer and coyote time indicators
        let debug_text = format!(
            "Jump Buffer: {:.2}\nCoyote Time: {:.2}\nVelocity: ({:.2}, {:.2})",
            self.jump_buffer, self.coyote_time, self.velocity.x, self.velocity.y
        );
        draw.text(font, &debug_text)
            .position(10.0, 10.0)
            .size(16.0)
            .color(Color::WHITE);

        // Draw MTV
        if let Some(mtv) = self.last_mtv {
            let mtv_end = center + mtv.to_vec2() * 100.0; // Scale up for visibility
            draw.line((center.x, center.y), (mtv_end.x, mtv_end.y))
                .width(2.0)
                .color(Color::BLUE);

            let mtv_text = format!("MTV: ({:.2}, {:.2})", mtv.0, mtv.1);
            draw.text(font, &mtv_text)
                .position(10.0, 50.0)
                .size(16.0)
                .color(Color::WHITE);
        }

        // Add vertex order visualization
        for (i, vertex) in self.polygon.vertices.iter().enumerate() {
            let pos = self.polygon.position.to_vec2() + vertex.to_vec2();
            draw.circle(3.0)
                .position(pos.x, pos.y)
                .color(Color::RED)
                .fill();
            draw.text(font, &i.to_string())
                .position(pos.x + 5.0, pos.y + 5.0)
                .size(12.0)
                .color(Color::WHITE);
        }
    }
}
