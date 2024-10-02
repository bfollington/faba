mod player;
mod tilemap;
use macroquad::prelude::*;
use player::Player;
use tilemap::{TileMap, TILE_SIZE};

#[macroquad::main("Cave Story Movement")]
async fn main() {
    let mut player = Player::new(50.0, 80.0);
    let mut jump_cooldown = 0.0;

    let mut tilemap = TileMap::new(20, 15); // 320x240 screen with 16x16 tiles
                                            // Set up some basic terrain
    for x in 0..20 {
        tilemap.set_tile(x, 14, true); // Ground
    }
    for y in 10..14 {
        tilemap.set_tile(10, y, true); // Vertical wall
    }

    loop {
        let dt = get_frame_time();

        clear_background(BLACK);

        // Handle input
        let left = is_key_down(KeyCode::Left);
        let right = is_key_down(KeyCode::Right);

        // Jump handling with cooldown
        jump_cooldown -= dt;
        if is_key_down(KeyCode::Space) && jump_cooldown <= 0.0 && player.on_ground {
            player.jump();
            jump_cooldown = 0.2; // 200ms cooldown
        }

        // Update player velocity based on input
        player.move_horizontal(left, right, dt);

        // Update player position and handle collisions
        player.update(&tilemap, dt);

        // Draw tilemap
        for y in 0..tilemap.height {
            for x in 0..tilemap.width {
                if tilemap.tiles[y][x] {
                    draw_rectangle(
                        x as f32 * TILE_SIZE,
                        y as f32 * TILE_SIZE,
                        TILE_SIZE,
                        TILE_SIZE,
                        GRAY,
                    );
                }
            }
        }

        // Draw player
        draw_rectangle(
            player.pos.x,
            player.pos.y,
            player.size.x,
            player.size.y,
            WHITE,
        );

        next_frame().await
    }
}
