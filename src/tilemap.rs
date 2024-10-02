use crate::player::Player;

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

    pub fn is_solid(&self, x: f32, y: f32) -> bool {
        matches!(self.get_tile_worldspace(x, y), TileType::Solid)
    }
}
