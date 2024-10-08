use notan::math::Mat3;
use notan::prelude::*;
use notan::{draw::*, math::Vec2};

mod gun;
mod player;
mod render;
mod soko;
mod textbox;
mod tilemap;
mod timer;
mod top_down;

use player::Player;
use render::PostProcessTarget;
use textbox::{Conversation, Textbox};
use tilemap::{TileMap, TileType, TILE_SIZE};

const GAME_WIDTH: u32 = 320;
const GAME_HEIGHT: u32 = 240;
const WINDOW_WIDTH: u32 = GAME_WIDTH * 2;
const WINDOW_HEIGHT: u32 = GAME_HEIGHT * 2;

#[derive(AppState)]
struct State {
    soko_player: soko::SokoPlayer,
    top_down: top_down::TopDownPlayer,
    tilemap: TileMap,
    jump_cooldown: f32,
    post_process: PostProcessTarget,
    conversation: Conversation,
    gun: gun::Gun,
    camera: Vec2,
    camera_shake: Vec2,
    shake_timer: f32,
}

#[notan_main]
fn main() -> Result<(), String> {
    let win_config = WindowConfig::new()
        .set_size(WINDOW_WIDTH, WINDOW_HEIGHT)
        .set_title("faba");

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

    // // Create a slope going up-right
    // for x in 5..9 {
    //     tilemap.set_tile(x, 13 - (x - 5), TileType::SlopeUpRight);
    // }

    // // Create a slope going up-left
    // for x in 12..17 {
    //     tilemap.set_tile(x, 9 + (x - 12), TileType::SlopeUpLeft);
    // }

    // Add some platforms
    for x in 2..7 {
        tilemap.set_tile(x, 10, TileType::Solid);
    }
    for x in 15..19 {
        tilemap.set_tile(x, 5, TileType::Solid);
    }

    // // Fill the area under the slopes with solid tiles
    // for x in 5..9 {
    //     for y in (14 - (x - 5))..14 {
    //         tilemap.set_tile(x, y, TileType::Solid);
    //     }
    // }
    // for x in 12..17 {
    //     for y in (10 + (x - 12))..14 {
    //         tilemap.set_tile(x, y, TileType::Solid);
    //     }
    // }

    let post_process = PostProcessTarget::new(gfx, GAME_WIDTH, GAME_HEIGHT);
    let soko_player = soko::SokoPlayer::new(0, 0);

    let top_down = top_down::TopDownPlayer::new(64., 64.);

    let mut conversation = Conversation::new(vec![
        textbox::Message::Text("This.".to_string()),
        textbox::Message::Text("This is a test.".to_string()),
        textbox::Message::Text("This is only a test.".to_string()),
        textbox::Message::Text("This is a test of the emergency broadcast system.".to_string()),
    ]);
    conversation.setup(gfx);

    State {
        soko_player,
        top_down,
        tilemap,
        jump_cooldown: 0.0,
        post_process,
        conversation,
        gun: gun::Gun::new(),
        camera: Vec2::new(0.0, 0.0),
        camera_shake: Vec2::new(0.0, 0.0),
        shake_timer: 0.0,
    }
}

fn update(app: &mut App, state: &mut State) {
    let dt = app.timer.delta_f32();
    let (mx, my) = app.mouse.position();

    let left = app.keyboard.was_pressed(KeyCode::Left);
    let right = app.keyboard.was_pressed(KeyCode::Right);
    let up = app.keyboard.was_pressed(KeyCode::Up);
    let down = app.keyboard.was_pressed(KeyCode::Down);

    let shoot = app.mouse.was_pressed(MouseButton::Left);

    let left_held = app.keyboard.is_down(KeyCode::A);
    let right_held = app.keyboard.is_down(KeyCode::D);
    let up_held = app.keyboard.is_down(KeyCode::W);
    let down_held = app.keyboard.is_down(KeyCode::S);
    let sprint = app.keyboard.is_down(KeyCode::LShift);
    let advance = app.keyboard.was_pressed(KeyCode::Space);

    if advance && state.conversation.textbox.finished_printing() {
        state.conversation.advance();
    }

    state.soko_player.update(dt, left, right, up, down);
    state
        .top_down
        .move_direction(up_held, down_held, left_held, right_held, sprint, dt);
    state.top_down.update(&state.tilemap, dt);
    state.gun.update(
        (state.top_down.pos.x, state.top_down.pos.y),
        ((mx / 2.) - state.camera.x, (my / 2.) - state.camera.y),
        dt,
    );
    state.conversation.update(dt);

    if shoot {
        state.shake_timer = 0.1;
        state.gun.shoot(state.top_down.pos.into());
    }

    if state.shake_timer > 0.0 {
        state.shake_timer -= dt;
    }

    // Set camera to a random position in a small range
    let shake_factor = if state.shake_timer > 0.0 { 6.0 } else { 0.0 };
    let target_x = -state.top_down.pos.x + GAME_WIDTH as f32 / 2.0;
    let target_y = -state.top_down.pos.y + GAME_HEIGHT as f32 / 2.0;
    let factor = 1.0 / dt; // Adjust this value to control the easing speed
    state.camera.x += (target_x - state.camera.x) / factor;
    state.camera.y += (target_y - state.camera.y) / factor;
    state.camera_shake.x = rand::random::<f32>() * shake_factor - shake_factor / 2.0;
    state.camera_shake.y = rand::random::<f32>() * shake_factor - shake_factor / 2.0;
}

fn draw(app: &mut App, gfx: &mut Graphics, state: &mut State) {
    let mut draw = gfx.create_draw();
    draw.clear(Color::BLACK);
    draw.transform().clear();
    draw.transform()
        .push(Mat3::from_translation(state.camera + state.camera_shake));

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

    // // Draw player
    state.soko_player.draw(&mut draw);
    state.top_down.draw(&mut draw);

    state.top_down.render_debug(&mut draw, &state.tilemap);
    state.gun.draw(&mut draw, state.top_down.pos.into());

    // Pop camera for UI
    draw.transform().pop();

    state.conversation.draw(&mut draw);

    // Render the game scene to the post-process texture
    gfx.render_to(&state.post_process.render_texture, &draw);

    // Apply post-processing and render to the screen
    let post_process_renderer = state
        .post_process
        .create_renderer(gfx, app.timer.delta_f32());
    gfx.render(&post_process_renderer);
}
