use notan::draw::*;
use notan::prelude::*;
use rapier2d::prelude::*;

pub const TILE_SIZE: f32 = 16.0;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SlopeType {
    LeftUp,
    RightUp,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum TileType {
    Empty,
    Solid,
    Slope(SlopeType),
}

pub struct TileMap {
    pub tiles: Vec<Vec<TileType>>,
    pub width: usize,
    pub height: usize,
}

impl TileMap {
    pub fn new(width: usize, height: usize) -> Self {
        let tiles = vec![vec![TileType::Empty; width]; height];
        TileMap {
            tiles,
            width,
            height,
        }
    }

    pub fn set_tile(&mut self, x: usize, y: usize, tile_type: TileType) {
        if x < self.width && y < self.height {
            self.tiles[y][x] = tile_type;
        }
    }

    pub fn get_tile(&self, x: usize, y: usize) -> TileType {
        if x < self.width && y < self.height {
            self.tiles[y][x]
        } else {
            TileType::Solid // Treat out-of-bounds as solid
        }
    }

    pub fn add_colliders(&self, colliders: &mut ColliderSet) {
        for y in 0..self.height {
            for x in 0..self.width {
                let position = vector![x as f32 * TILE_SIZE, y as f32 * TILE_SIZE];
                match self.get_tile(x, y) {
                    TileType::Solid => {
                        let collider = ColliderBuilder::cuboid(TILE_SIZE / 2.0, TILE_SIZE / 2.0)
                            .translation(position + vector![TILE_SIZE / 2.0, TILE_SIZE / 2.0])
                            .build();
                        colliders.insert(collider);
                    }
                    TileType::Slope(slope_type) => {
                        let vertices = match slope_type {
                            SlopeType::LeftUp => vec![
                                point![0.0, TILE_SIZE],
                                point![TILE_SIZE, 0.0],
                                point![TILE_SIZE, TILE_SIZE],
                            ],
                            SlopeType::RightUp => vec![
                                point![0.0, 0.0],
                                point![TILE_SIZE, TILE_SIZE],
                                point![0.0, TILE_SIZE],
                            ],
                        };
                        let collider = ColliderBuilder::polyline(vertices, None)
                            .translation(position)
                            .build();
                        colliders.insert(collider);
                    }
                    TileType::Empty => {}
                }
            }
        }
    }

    pub fn debug_render(&self, draw: &mut Draw, font: &Font) {
        for y in 0..self.height {
            for x in 0..self.width {
                let pos = (x as f32 * TILE_SIZE, y as f32 * TILE_SIZE);
                match self.get_tile(x, y) {
                    TileType::Solid => {
                        draw.rect(pos, (TILE_SIZE, TILE_SIZE))
                            .color(Color::from_rgb(0.0, 1.0, 0.0))
                            .stroke(1.0);
                    }
                    TileType::Slope(slope_type) => {
                        let mut path = draw.path();
                        path.move_to(pos.0, pos.1 + TILE_SIZE);
                        path.line_to(pos.0 + TILE_SIZE, pos.1 + TILE_SIZE);
                        match slope_type {
                            SlopeType::LeftUp => path.line_to(pos.0 + TILE_SIZE, pos.1),
                            SlopeType::RightUp => path.line_to(pos.0, pos.1),
                        };
                        path.close();
                        path.stroke(1.0).color(Color::from_rgb(1.0, 1.0, 0.0));
                    }
                    TileType::Empty => {}
                }
            }
        }
    }
}
