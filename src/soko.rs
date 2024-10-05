use crate::tilemap::{TileMap, TileType, TILE_SIZE};
use crate::timer::Timer;
use notan::draw::*;
use notan::math::Vec2;
use notan::prelude::*;
use simple_easing::*;
use spring_motion::*;

enum Direction {
    Up,
    Down,
    Left,
    Right,
}

pub struct SokoPlayer {
    pub x: i32,
    pub y: i32,
    pub render_position: Vec2,
    pub direction: Direction,
    render_spring: SpringSystem<Vec2>,
    movement_animation: Timer,
    landing_animation: Timer,
}

impl SokoPlayer {
    pub fn new(x: i32, y: i32) -> Self {
        SokoPlayer {
            x,
            y,
            render_position: Vec2::new(x as f32, y as f32),
            render_spring: SpringSystem::new(
                SpringDescription {
                    frequency: 3.,
                    damping: 0.4,
                    initial_response: -0.1,
                },
                Vec2::splat(0.),
                Vec2::splat(0.),
            ),
            direction: Direction::Right,
            movement_animation: Timer::new(0.2),
            landing_animation: Timer::new(0.2),
        }
    }

    pub fn move_player(&mut self, dx: i32, dy: i32) {
        self.x += dx;
        self.y += dy;

        self.movement_animation.reset();
    }

    pub fn update(&mut self, dt: f32, left: bool, right: bool, up: bool, down: bool) {
        if left {
            self.move_player(-1, 0);
            self.direction = Direction::Left;
        } else if right {
            self.move_player(1, 0);
            self.direction = Direction::Right;
        } else if up {
            self.move_player(0, -1);
            self.direction = Direction::Up;
        } else if down {
            self.move_player(0, 1);
            self.direction = Direction::Down;
        }

        self.render_position = self
            .render_spring
            .step_clamped(dt, Vec2::new(self.x as f32, self.y as f32));

        self.movement_animation.update(dt);
        self.landing_animation.update(dt);
    }

    pub fn draw(&self, draw: &mut Draw) {
        let pos = self.render_position;

        // let scale = 0.5 + 0.5 * cubic_out(self.movement_animation.progress());
        let rotation = match self.direction {
            Direction::Up => -1.0,
            Direction::Down => 1.0,
            Direction::Left => -1.0,
            Direction::Right => 1.0,
        };
        let rotation = 0.1 * rotation * (1.0 - cubic_out(self.movement_animation.progress()));

        let scale =
            if self.movement_animation.is_finished() && !self.landing_animation.is_finished() {
                let t = self.landing_animation.progress();
                let t = t * t;
                0.8 + 0.2 * t * (2.0 - t) // Starts at 0.8 and animates back to 1.0
            } else if !self.movement_animation.is_finished() {
                let t = self.movement_animation.progress();
                let t = t * t;
                -0.5 * t * (t - 1.0) + 1.0
            } else {
                1.0
            };

        let width = TILE_SIZE * scale;
        let height = TILE_SIZE * scale;

        let offset = {
            let t = self.movement_animation.progress();
            let t = t * t;
            -32.0 * t * (t - 1.0)
        };

        draw.rect((0., 0.), (width, height))
            .color(Color::WHITE)
            .rotate(rotation)
            .translate(
                pos.x * TILE_SIZE - width / 2.0,
                pos.y * TILE_SIZE - height / 2.0 - offset,
            );
    }
}
