use crate::player::Player;

pub const TILE_SIZE: f32 = 16.0;

pub struct TileMap {
    pub tiles: Vec<Vec<bool>>, // true for solid, false for empty
    pub width: usize,
    pub height: usize,
}

impl TileMap {
    pub fn new(width: usize, height: usize) -> Self {
        let tiles = vec![vec![false; width]; height];
        TileMap {
            tiles,
            width,
            height,
        }
    }

    pub fn set_tile(&mut self, x: usize, y: usize, solid: bool) {
        if x < self.width && y < self.height {
            self.tiles[y][x] = solid;
        }
    }

    pub fn is_solid(&self, x: f32, y: f32) -> bool {
        let tile_x = (x / TILE_SIZE) as usize;
        let tile_y = (y / TILE_SIZE) as usize;
        if tile_x < self.width && tile_y < self.height {
            self.tiles[tile_y][tile_x]
        } else {
            true // Treat out-of-bounds as solid
        }
    }
}
