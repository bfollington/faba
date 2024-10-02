use notan::draw::*;
use notan::math::Vec2;
use notan::prelude::*;
use rapier2d::prelude::*;

mod player;
mod tilemap;

use player::Player;
use tilemap::{SlopeType, TileMap, TileType, TILE_SIZE};

const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;

#[derive(AppState)]
struct State {
    player: Player,
    tilemap: TileMap,
    font: Font,
    fps: f32,
    rigid_body_set: RigidBodySet,
    collider_set: ColliderSet,
    physics_pipeline: PhysicsPipeline,
    island_manager: IslandManager,
    broad_phase: BroadPhase,
    narrow_phase: NarrowPhase,
    impulse_joint_set: ImpulseJointSet,
    multibody_joint_set: MultibodyJointSet,
    ccd_solver: CCDSolver,
    physics_hooks: (),
    event_handler: (),
    query_pipeline: QueryPipeline,
}

#[notan_main]
fn main() -> Result<(), String> {
    let win = WindowConfig::new()
        .set_size(WINDOW_WIDTH, WINDOW_HEIGHT)
        .set_title("Cave Story Movement");

    notan::init_with(setup)
        .add_config(win)
        .add_config(DrawConfig)
        .update(update)
        .draw(draw)
        .build()
}

fn setup(_app: &mut App, gfx: &mut Graphics) -> State {
    let mut rigid_body_set = RigidBodySet::new();
    let mut collider_set = ColliderSet::new();

    let player = Player::new(
        50.0,
        450.0,
        24.,
        24.,
        &mut rigid_body_set,
        &mut collider_set,
    );
    let mut tilemap = TileMap::new(40, 30);

    // Set up ground
    for x in 0..40 {
        tilemap.set_tile(x, 29, TileType::Solid);
    }

    // Create large rectangular platforms
    for x in 2..10 {
        for y in 20..22 {
            tilemap.set_tile(x, y, TileType::Solid);
        }
    }

    for x in 15..25 {
        for y in 15..17 {
            tilemap.set_tile(x, y, TileType::Solid);
        }
    }

    for x in 30..38 {
        for y in 10..12 {
            tilemap.set_tile(x, y, TileType::Solid);
        }
    }

    // Add some smaller platforms
    for x in 12..14 {
        tilemap.set_tile(x, 25, TileType::Solid);
    }

    for x in 26..28 {
        tilemap.set_tile(x, 20, TileType::Solid);
    }

    // Add slopes on the ground layer
    tilemap.set_tile(20, 28, TileType::Slope(SlopeType::LeftUp));
    tilemap.set_tile(21, 28, TileType::Slope(SlopeType::LeftUp));
    tilemap.set_tile(22, 27, TileType::Slope(SlopeType::LeftUp));
    tilemap.set_tile(23, 27, TileType::Slope(SlopeType::LeftUp));
    tilemap.set_tile(24, 26, TileType::Slope(SlopeType::LeftUp));
    tilemap.set_tile(25, 26, TileType::Slope(SlopeType::LeftUp));

    tilemap.set_tile(30, 28, TileType::Slope(SlopeType::RightUp));
    tilemap.set_tile(31, 28, TileType::Slope(SlopeType::RightUp));
    tilemap.set_tile(32, 27, TileType::Slope(SlopeType::RightUp));
    tilemap.set_tile(33, 27, TileType::Slope(SlopeType::RightUp));
    tilemap.set_tile(34, 26, TileType::Slope(SlopeType::RightUp));
    tilemap.set_tile(35, 26, TileType::Slope(SlopeType::RightUp));

    // Add tilemap colliders
    tilemap.add_colliders(&mut collider_set);

    let font = gfx
        .create_font(include_bytes!("../assets/Ubuntu-B.ttf"))
        .unwrap();

    let fps = _app.timer.fps();

    State {
        player,
        tilemap,
        font,
        fps,
        rigid_body_set,
        collider_set,
        physics_pipeline: PhysicsPipeline::new(),
        island_manager: IslandManager::new(),
        broad_phase: BroadPhase::new(),
        narrow_phase: NarrowPhase::new(),
        impulse_joint_set: ImpulseJointSet::new(),
        multibody_joint_set: MultibodyJointSet::new(),
        ccd_solver: CCDSolver::new(),
        physics_hooks: (),
        event_handler: (),
        query_pipeline: QueryPipeline::new(),
    }
}

fn update(app: &mut App, state: &mut State) {
    let dt = app.timer.delta_f32();

    // Handle input
    let move_left = app.keyboard.is_down(KeyCode::Left);
    let move_right = app.keyboard.is_down(KeyCode::Right);
    let jump_pressed = app.keyboard.was_pressed(KeyCode::Space);

    // Update player
    state
        .player
        .set_movement(move_left, move_right, &mut state.rigid_body_set);
    if jump_pressed {
        state.player.jump(&mut state.rigid_body_set);
    }

    // Step the physics world
    let gravity = vector![0.0, 9.81 * 10.0];
    let integration_parameters = IntegrationParameters::default();
    state
        .query_pipeline
        .update(&state.rigid_body_set, &state.collider_set);
    state.physics_pipeline.step(
        &gravity,
        &integration_parameters,
        &mut state.island_manager,
        &mut state.broad_phase,
        &mut state.narrow_phase,
        &mut state.rigid_body_set,
        &mut state.collider_set,
        &mut state.impulse_joint_set,
        &mut state.multibody_joint_set,
        &mut state.ccd_solver,
        Some(&mut state.query_pipeline),
        &state.physics_hooks,
        &state.event_handler,
    );

    state.player.update(
        &state.rigid_body_set,
        &state.collider_set,
        &state.query_pipeline,
    );
}

fn draw(gfx: &mut Graphics, state: &mut State) {
    let mut draw = gfx.create_draw();
    draw.clear(Color::BLACK);

    draw_tilemap(&mut draw, &state.tilemap);
    &state.player.render(&mut draw, &state.rigid_body_set);

    // Debug rendering
    state
        .player
        .debug_render(&mut draw, &state.font, &state.rigid_body_set);
    state.tilemap.debug_render(&mut draw, &state.font);

    // Draw FPS and player position
    draw.text(&state.font, &format!("FPS: {}", &state.fps))
        .position(10.0, 20.0)
        .size(20.0)
        .color(Color::WHITE);

    let pos = state.player.position(&state.rigid_body_set);
    draw.text(&state.font, &format!("Pos: ({:.2}, {:.2})", pos.x, pos.y))
        .position(10.0, 40.0)
        .size(20.0)
        .color(Color::WHITE);

    gfx.render(&draw);
}

fn draw_tilemap(draw: &mut Draw, tilemap: &TileMap) {
    for y in 0..tilemap.height {
        for x in 0..tilemap.width {
            match tilemap.get_tile(x, y) {
                TileType::Solid => {
                    draw.rect(
                        (x as f32 * TILE_SIZE, y as f32 * TILE_SIZE),
                        (TILE_SIZE, TILE_SIZE),
                    )
                    .color(Color::GRAY);
                }
                TileType::Slope(slope_type) => {
                    let (x, y) = (x as f32 * TILE_SIZE, y as f32 * TILE_SIZE);
                    match slope_type {
                        SlopeType::LeftUp => {
                            draw.triangle(
                                (x, y + TILE_SIZE),
                                (x + TILE_SIZE, y + TILE_SIZE),
                                (x + TILE_SIZE, y),
                            )
                            .color(Color::GRAY);
                        }
                        SlopeType::RightUp => {
                            draw.triangle(
                                (x, y + TILE_SIZE),
                                (x + TILE_SIZE, y + TILE_SIZE),
                                (x, y),
                            )
                            .color(Color::GRAY);
                        }
                    }
                }
                TileType::Empty => {}
            }
        }
    }
}
