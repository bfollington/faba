mod player;
mod tilemap;
use macroquad::prelude::*;
use player::Player;
use tilemap::{SlopeType, TileMap, TileType, TILE_SIZE};

#[macroquad::main("Cave Story Movement")]
async fn main() {
    let mut player = Player::new(50.0, 80.0);
    let mut tilemap = TileMap::new(20, 15); // Initialize your tilemap here

    // Set up some tiles for testing
    for x in 0..20 {
        tilemap.set_tile(x, 14, TileType::Solid); // Ground
    }
    tilemap.set_tile(10, 13, TileType::Solid); // Wall
    tilemap.set_tile(5, 13, TileType::Slope(SlopeType::RightUp)); // Slope

    loop {
        let dt = get_frame_time();

        // Handle input
        let move_left = is_key_down(KeyCode::Left);
        let move_right = is_key_down(KeyCode::Right);
        let jump_pressed = is_key_pressed(KeyCode::Space);
        let jump_released = is_key_released(KeyCode::Space);

        // Update player
        player.set_movement(move_left, move_right);
        if jump_pressed {
            player.jump();
        }
        if jump_released {
            player.release_jump();
        }
        player.update(&tilemap, dt);

        // Clear the screen and draw
        clear_background(BLACK);
        draw_tilemap(&tilemap);
        draw_player(&player);

        // Debug rendering
        player.debug_render(&tilemap);

        // Draw FPS and player position
        draw_text(&format!("FPS: {}", get_fps()), 10.0, 20.0, 20.0, WHITE);
        draw_text(
            &format!("Pos: ({:.2}, {:.2})", player.pos.x, player.pos.y),
            10.0,
            40.0,
            20.0,
            WHITE,
        );

        next_frame().await
    }
}

fn draw_tilemap(tilemap: &TileMap) {
    for y in 0..tilemap.height {
        for x in 0..tilemap.width {
            match tilemap.get_tile(x as usize, y as usize) {
                TileType::Solid => draw_rectangle(
                    x as f32 * TILE_SIZE,
                    y as f32 * TILE_SIZE,
                    TILE_SIZE,
                    TILE_SIZE,
                    GRAY,
                ),
                TileType::Slope(slope_type) => {
                    let (x, y) = (x as f32 * TILE_SIZE, y as f32 * TILE_SIZE);
                    match slope_type {
                        SlopeType::LeftUp => draw_triangle(
                            Vec2::new(x, y + TILE_SIZE),
                            Vec2::new(x + TILE_SIZE, y + TILE_SIZE),
                            Vec2::new(x + TILE_SIZE, y),
                            GRAY,
                        ),
                        SlopeType::RightUp => draw_triangle(
                            Vec2::new(x, y + TILE_SIZE),
                            Vec2::new(x + TILE_SIZE, y + TILE_SIZE),
                            Vec2::new(x, y),
                            GRAY,
                        ),
                    }
                }
                TileType::Empty => {}
            }
        }
    }
}

fn draw_player(player: &Player) {
    draw_rectangle(
        player.pos.x,
        player.pos.y,
        player.size.x,
        player.size.y,
        WHITE,
    );
}
