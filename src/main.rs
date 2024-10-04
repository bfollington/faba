use notan::draw::*;
use notan::prelude::*;

mod player;
mod render;
mod tilemap;

use player::Player;
use render::PostProcessTarget;
use tilemap::{TileMap, TileType, TILE_SIZE};

const GAME_WIDTH: u32 = 320;
const GAME_HEIGHT: u32 = 240;
const WINDOW_WIDTH: u32 = GAME_WIDTH * 2;
const WINDOW_HEIGHT: u32 = GAME_HEIGHT * 2;

#[derive(AppState)]
struct State {
    player: Player,
    tilemap: TileMap,
    jump_cooldown: f32,
    post_process: PostProcessTarget,
}

#[notan_main]
fn main() -> Result<(), String> {
    let win_config = WindowConfig::new()
        .set_size(WINDOW_WIDTH, WINDOW_HEIGHT)
        .set_title("Cave Story Movement with Slopes");

    notan::init_with(setup)
        .add_config(win_config)
        .add_config(DrawConfig)
        .update(update)
        .draw(draw)
        .build()
}

fn setup(_app: &mut App, gfx: &mut Graphics) -> State {
    let player = Player::new(50.0, 80.0);
    let mut tilemap = TileMap::new(20, 15); // 320x240 screen with 16x16 tiles

    // Set up an example level with slopes
    for x in 0..20 {
        tilemap.set_tile(x, 14, TileType::Solid); // Ground
    }

    // Create a slope going up-right
    for x in 5..9 {
        tilemap.set_tile(x, 13 - (x - 5), TileType::SlopeUpRight);
    }

    // Create a slope going up-left
    for x in 12..17 {
        tilemap.set_tile(x, 9 + (x - 12), TileType::SlopeUpLeft);
    }

    // Add some platforms
    for x in 2..7 {
        tilemap.set_tile(x, 10, TileType::Solid);
    }
    for x in 15..19 {
        tilemap.set_tile(x, 5, TileType::Solid);
    }

    // Fill the area under the slopes with solid tiles
    for x in 5..9 {
        for y in (14 - (x - 5))..14 {
            tilemap.set_tile(x, y, TileType::Solid);
        }
    }
    for x in 12..17 {
        for y in (10 + (x - 12))..14 {
            tilemap.set_tile(x, y, TileType::Solid);
        }
    }

    let post_process = PostProcessTarget::new(gfx, GAME_WIDTH, GAME_HEIGHT);

    State {
        player,
        tilemap,
        jump_cooldown: 0.0,
        post_process,
    }
}

fn update(app: &mut App, state: &mut State) {
    let dt = app.timer.delta_f32();

    // Handle input
    let left = app.keyboard.is_down(KeyCode::Left);
    let right = app.keyboard.is_down(KeyCode::Right);
    let sprint = app.keyboard.is_down(KeyCode::LShift);
    let jump_pressed = app.keyboard.is_down(KeyCode::Space);

    // Update player velocity based on input
    state.player.move_horizontal(left, right, sprint, dt);

    // Handle jumping
    if app.keyboard.was_pressed(KeyCode::Space) {
        state.player.jump();
    }
    if app.keyboard.was_released(KeyCode::Space) {
        state.player.cancel_jump();
    }

    // Update player position and handle collisions
    state.player.update(&state.tilemap, dt, jump_pressed);
}

fn draw(app: &mut App, gfx: &mut Graphics, state: &mut State) {
    let mut draw = gfx.create_draw();
    draw.clear(Color::BLACK);

    // Draw tilemap
    for y in 0..state.tilemap.height {
        for x in 0..state.tilemap.width {
            match state.tilemap.tiles[y][x] {
                TileType::Solid => {
                    draw.rect(
                        (x as f32 * TILE_SIZE, y as f32 * TILE_SIZE),
                        (TILE_SIZE, TILE_SIZE),
                    )
                    .color(Color::GRAY);
                }
                TileType::SlopeUpRight => {
                    draw.triangle(
                        (x as f32 * TILE_SIZE, (y + 1) as f32 * TILE_SIZE),
                        ((x + 1) as f32 * TILE_SIZE, (y + 1) as f32 * TILE_SIZE),
                        ((x + 1) as f32 * TILE_SIZE, y as f32 * TILE_SIZE),
                    )
                    .color(Color::BLUE);
                }
                TileType::SlopeUpLeft => {
                    draw.triangle(
                        (x as f32 * TILE_SIZE, (y + 1) as f32 * TILE_SIZE),
                        ((x + 1) as f32 * TILE_SIZE, (y + 1) as f32 * TILE_SIZE),
                        (x as f32 * TILE_SIZE, y as f32 * TILE_SIZE),
                    )
                    .color(Color::GREEN);
                }
                TileType::Empty => {}
            }
        }
    }

    state.player.render_debug(&mut draw, &state.tilemap);

    // Draw player
    draw.rect(
        (state.player.pos.x, state.player.pos.y),
        (state.player.size.x, state.player.size.y),
    )
    .color(Color::WHITE);

    // Render the game scene to the post-process texture
    gfx.render_to(&state.post_process.render_texture, &draw);

    // Apply post-processing and render to the screen
    let post_process_renderer = state
        .post_process
        .create_renderer(gfx, app.timer.delta_f32());
    gfx.render(&post_process_renderer);
}
