pub const TILE_SIZE: f32 = 16.0;

#[derive(Clone, Copy, PartialEq)]
pub enum TileType {
    Empty,
    Solid,
    SlopeUpRight,
    SlopeUpLeft,
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

    pub fn get_tile_type(&self, x: f32, y: f32) -> TileType {
        let tile_x = (x / TILE_SIZE) as usize;
        let tile_y = (y / TILE_SIZE) as usize;
        if tile_x < self.width && tile_y < self.height {
            self.tiles[tile_y][tile_x]
        } else {
            TileType::Solid // Treat out-of-bounds as solid
        }
    }

    pub fn is_pixel_solid(&self, x: f32, y: f32) -> bool {
        let tile_type = self.get_tile_type(x, y);
        let px = (x % TILE_SIZE) as u32;
        let py = (y % TILE_SIZE) as u32;

        match tile_type {
            TileType::Solid => true,
            TileType::SlopeUpRight => px >= TILE_SIZE as u32 - py,
            TileType::SlopeUpLeft => px < py,
            TileType::Empty => false,
        }
    }

    pub fn is_tile_type(&self, x: f32, y: f32, tile_type: TileType) -> bool {
        let tile_x = (x / TILE_SIZE) as usize;
        let tile_y = (y / TILE_SIZE) as usize;
        if tile_x < self.width && tile_y < self.height {
            self.tiles[tile_y][tile_x] == tile_type
        } else {
            false
        }
    }
}
