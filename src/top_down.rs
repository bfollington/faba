use crate::tilemap::{TileMap, TileType, TILE_SIZE};
use notan::draw::*;
use notan::math::{Mat3, Rect, Vec2};
use notan::prelude::*;

pub struct TopDownPlayer {
    pub pos: Vec2,
    pub velocity: Vec2,
    pub temp_velocity: Vec2,
    pub size: Vec2,
    pub shape: Rect,
    pub render_matrix: Mat3,
    pub acceleration: Vec2,
    pub friction: Vec2,
    pub max_speed: Vec2,
    pub moved_amount: Vec2,
    collision_types: Vec<TileType>,
    sprint_speed_multiplier: f32,
}

impl TopDownPlayer {
    pub fn new(x: f32, y: f32) -> Self {
        let mut shape = Rectangle::new((0.0, 0.0), (8.0, 8.0));
        shape.translate(0., 4.);

        TopDownPlayer {
            pos: Vec2::new(x, y),
            velocity: Vec2::ZERO,
            temp_velocity: Vec2::ZERO,
            size: Vec2::new(8.0, 8.0),
            shape: Rect {
                x: 0.0,
                y: 0.0,
                width: 8.0,
                height: 16.0,
            },
            render_matrix: Mat3::from_translation(Vec2::new(0., -8.)),
            acceleration: Vec2::new(420.0, 420.0),
            friction: Vec2::new(1.15, 1.15),
            max_speed: Vec2::new(2., 2.),
            moved_amount: Vec2::ZERO,
            collision_types: vec![TileType::Solid],
            sprint_speed_multiplier: 2.,
        }
    }

    pub fn update(&mut self, tilemap: &TileMap, dt: f32) {
        self.velocity /= self.friction;

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
            self.move_x(tilemap, horizontal_direction);
            self.temp_velocity.x -= 1.0;
        }
    }

    fn move_x(&mut self, tilemap: &TileMap, dir: i32) {
        self.moved_amount.x += 1.0;
        let new_x = self.pos.x + dir as f32;

        if !self.collide(tilemap, new_x, self.pos.y) {
            self.pos.x = new_x;
        } else {
            self.velocity.x = 0.0;
        }
    }

    fn move_y(&mut self, tilemap: &TileMap, dir: i32) {
        self.moved_amount.y += 1.0;
        let new_y = self.pos.y + dir as f32;

        if !self.collide(tilemap, self.pos.x, new_y) {
            self.pos.y = new_y;
        } else {
            self.velocity.y = 0.0;
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
                if tile_type == TileType::Solid {
                    return true;
                }
            }
        }
        false
    }

    fn hit_wall(&mut self) {
        self.temp_velocity = Vec2::ZERO;
        self.velocity = Vec2::ZERO;
    }

    fn clamp_speed(&mut self) {
        let max_speed = self.max_speed * self.sprint_speed_multiplier;
        self.velocity.x = self.velocity.x.clamp(-max_speed.x, max_speed.x);
        self.velocity.y = self.velocity.y.clamp(-max_speed.y, max_speed.y);
    }

    pub fn move_direction(
        &mut self,
        up: bool,
        down: bool,
        left: bool,
        right: bool,
        sprint: bool,
        dt: f32,
    ) {
        let mut direction = Vec2::ZERO;
        if up {
            direction.y -= 1.0;
        }
        if down {
            direction.y += 1.0;
        }
        if left {
            direction.x -= 1.0;
        }
        if right {
            direction.x += 1.0;
        }

        if direction != Vec2::ZERO {
            direction = direction.normalize();
        }

        let acceleration = if sprint {
            self.acceleration * self.sprint_speed_multiplier
        } else {
            self.acceleration
        };

        self.velocity += direction * acceleration * dt;
    }

    pub fn draw(&self, draw: &mut Draw) {
        let pos = self.pos;

        draw.transform().push(self.render_matrix);

        draw.rect((0., 0.), (self.shape.width, self.shape.height))
            .color(Color::WHITE)
            .translate(pos.x + self.shape.x, pos.y + self.shape.y);

        draw.transform().pop();
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

        // Draw velocity vector
        draw.line(
            (
                self.pos.x + self.size.x / 2.0,
                self.pos.y + self.size.y / 2.0,
            ),
            (
                self.pos.x + self.size.x / 2.0 + self.velocity.x * 10.0,
                self.pos.y + self.size.y / 2.0 + self.velocity.y * 10.0,
            ),
        )
        .color(Color::RED);

        // Render tiles that the player is colliding with
        for y in (self.pos.y as i32 - 1..=(self.pos.y + self.size.y) as i32 + 1) {
            for x in (self.pos.x as i32 - 1..=(self.pos.x + self.size.x) as i32 + 1) {
                let tile_type = tilemap.get_tile_type(x as f32, y as f32);
                if tile_type == TileType::Solid {
                    draw.rect(
                        (x as f32 * TILE_SIZE, y as f32 * TILE_SIZE),
                        (TILE_SIZE, TILE_SIZE),
                    )
                    .stroke_color(Color::BLUE);
                }
            }
        }
    }
}
