use notan::draw::*;
use notan::math::Vec2;
use notan::prelude::*;

mod player;
mod tilemap;

use player::Player;
use tilemap::{TileMap, TILE_SIZE};

const WINDOW_WIDTH: u32 = 320;
const WINDOW_HEIGHT: u32 = 240;

#[derive(AppState)]
struct State {
    player: Player,
    tilemap: TileMap,
    jump_cooldown: f32,
}

#[notan_main]
fn main() -> Result<(), String> {
    let win_config = WindowConfig::new()
        .set_size(WINDOW_WIDTH, WINDOW_HEIGHT)
        .set_title("Cave Story Movement");

    notan::init_with(setup)
        .add_config(win_config)
        .add_config(DrawConfig)
        .update(update)
        .draw(draw)
        .build()
}

fn setup(_app: &mut App, gfx: &mut Graphics) -> State {
    let mut player = Player::new(50.0, 80.0);
    let mut tilemap = TileMap::new(20, 15); // 320x240 screen with 16x16 tiles

    // Set up some basic terrain
    for x in 0..20 {
        tilemap.set_tile(x, 14, true); // Ground
    }
    for y in 10..14 {
        tilemap.set_tile(10, y, true); // Vertical wall
    }

    State {
        player,
        tilemap,
        jump_cooldown: 0.0,
    }
}

fn update(app: &mut App, state: &mut State) {
    let dt = app.timer.delta_f32();

    // Handle input
    let left = app.keyboard.is_down(KeyCode::Left);
    let right = app.keyboard.is_down(KeyCode::Right);

    // Jump handling with cooldown
    state.jump_cooldown -= dt;
    if app.keyboard.is_down(KeyCode::Space) && state.jump_cooldown <= 0.0 && state.player.on_ground
    {
        state.player.jump();
        state.jump_cooldown = 0.2; // 200ms cooldown
    }

    // Update player velocity based on input
    state.player.move_horizontal(left, right, dt);

    // Update player position and handle collisions
    state.player.update(&state.tilemap, dt);
}

fn draw(gfx: &mut Graphics, state: &mut State) {
    let mut draw = gfx.create_draw();
    draw.clear(Color::BLACK);

    // Draw tilemap
    for y in 0..state.tilemap.height {
        for x in 0..state.tilemap.width {
            if state.tilemap.tiles[y][x] {
                draw.rect(
                    (x as f32 * TILE_SIZE, y as f32 * TILE_SIZE),
                    (TILE_SIZE, TILE_SIZE),
                )
                .color(Color::GRAY);
            }
        }
    }

    // Draw player
    draw.rect(
        (state.player.pos.x, state.player.pos.y),
        (state.player.size.x, state.player.size.y),
    )
    .color(Color::WHITE);

    gfx.render(&draw);
}
