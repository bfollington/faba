use notan::draw::*;
use notan::math::{Rect, Vec2};
use notan::prelude::*;

use sepax2d::aabb::AABB;
use sepax2d::capsule::Capsule;
use sepax2d::circle::Circle;
use sepax2d::polygon::Polygon;

use crate::vertices::{ToTuple, ToVec2, Vertices};

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

    pub fn get_tile_worldspace(&self, x: f32, y: f32) -> TileType {
        let tile_x = (x / TILE_SIZE) as usize;
        let tile_y = (y / TILE_SIZE) as usize;
        self.get_tile(tile_x, tile_y)
    }

    pub fn collision_mask(&self) -> Vec<Polygon> {
        let mut polygons = Vec::new();
        for y in 0..self.height {
            for x in 0..self.width {
                match self.get_tile(x, y) {
                    TileType::Solid => {
                        let pos = (x as f32 * TILE_SIZE, y as f32 * TILE_SIZE);

                        polygons.push(Polygon::from_vertices(
                            (0., 0.),
                            Rect {
                                x: pos.0,
                                y: pos.1,
                                width: TILE_SIZE,
                                height: TILE_SIZE,
                            }
                            .vertices(),
                        ));
                    }
                    TileType::Slope(slope_type) => {
                        let pos = (x as f32 * TILE_SIZE, y as f32 * TILE_SIZE);
                        let vertices = match slope_type {
                            SlopeType::LeftUp => {
                                vec![
                                    (pos.0, pos.1 + TILE_SIZE),
                                    (pos.0 + TILE_SIZE, pos.1 + TILE_SIZE),
                                    (pos.0 + TILE_SIZE, pos.1),
                                ]
                            }
                            SlopeType::RightUp => {
                                vec![
                                    (pos.0, pos.1 + TILE_SIZE),
                                    (pos.0 + TILE_SIZE, pos.1 + TILE_SIZE),
                                    (pos.0, pos.1),
                                ]
                            }
                        };
                        polygons.push(Polygon::from_vertices((0., 0.), vertices));
                    }
                    TileType::Empty => {}
                }
            }
        }
        polygons
    }

    pub fn debug_render(&self, draw: &mut Draw, font: &Font) {
        for polygon in self.collision_mask() {
            let mut path = draw.path();

            path.move_to(
                polygon.vertices[0].0 + polygon.position.0,
                polygon.vertices[0].1 + polygon.position.1,
            );
            for vertex in polygon.vertices.iter().skip(1) {
                path.line_to(vertex.0 + polygon.position.0, vertex.1 + polygon.position.1);
            }
            path.close();
            path.color(Color::from_rgb(0.0, 1.0, 0.0)).fill();
        }
    }
}
