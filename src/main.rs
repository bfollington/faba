use notan::draw::*;
use notan::math::{Mat4, Vec2, Vec3};
use notan::prelude::*;

mod player;
mod tilemap;

use player::Player;
use tilemap::{TileMap, TileType, TILE_SIZE};

const WINDOW_WIDTH: u32 = 320;
const WINDOW_HEIGHT: u32 = 240;

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

fn setup(app: &mut App, gfx: &mut Graphics) -> State {
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

    let post_process = PostProcessTarget::new(gfx, WINDOW_WIDTH, WINDOW_HEIGHT);

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

    gfx.render(&draw);

    // Render the game scene to the post-process texture
    gfx.render_to(&state.post_process.render_texture, &draw);

    // Apply post-processing and render to the screen
    let post_process_renderer = state
        .post_process
        .create_renderer(gfx, app.timer.delta_f32());
    gfx.render(&post_process_renderer);
}

// Post-processing implementation

//language=glsl
const IMAGE_VERTEX: ShaderSource = notan::vertex_shader! {
    r#"
    #version 450
    layout(location = 0) in vec3 a_position;
    layout(location = 1) in vec2 a_texcoord;
    layout(location = 0) out vec2 v_texcoord;
    void main() {
        v_texcoord = a_texcoord;
        gl_Position = vec4(a_position, 1.0);
    }
    "#
};

//language=glsl
const PIXEL_INVERT_FRAGMENT: ShaderSource = notan::fragment_shader! {
    r#"
    #version 450
    precision mediump float;
    layout(location = 0) out vec4 outColor;
    layout(location = 0) in vec2 v_texcoord;
    layout(binding = 0) uniform sampler2D u_texture;
    layout(set = 0, binding = 0) uniform Locals {
        vec2 u_tex_size;
        float u_value;
    };
    void main() {
        vec2 size = vec2(u_value, u_value);
        vec2 coord = fract(v_texcoord) * u_tex_size;
        coord = floor(coord/size) * size;
        vec4 tex_color = texture(u_texture, coord / u_tex_size);
        float red = tex_color.r + ((1.0 - tex_color.r) * abs(sin(u_value)));
        float green = tex_color.g + ((1.0 - tex_color.g) * abs(sin(u_value)));
        float blue = tex_color.b + ((1.0 - tex_color.b) * abs(sin(u_value)));
        outColor = vec4(red, green, blue, tex_color.a);
    }
    "#
};

struct PostProcessTarget {
    render_texture: RenderTexture,
    pipeline: Pipeline,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    uniform_buffer: Buffer,
    value: f32,
}

impl PostProcessTarget {
    fn new(gfx: &mut Graphics, width: u32, height: u32) -> Self {
        let render_texture = gfx
            .create_render_texture(width, height)
            .with_depth()
            .build()
            .unwrap();

        let vertex_info = VertexInfo::new()
            .attr(0, VertexFormat::Float32x3)
            .attr(1, VertexFormat::Float32x2);

        let pipeline = gfx
            .create_pipeline()
            .from(&IMAGE_VERTEX, &PIXEL_INVERT_FRAGMENT)
            .with_color_blend(BlendMode::NORMAL)
            .with_vertex_info(&vertex_info)
            .with_texture_location(0, "u_texture")
            .build()
            .unwrap();

        #[rustfmt::skip]
        let vertices = [
            1.0,  1.0, 0.0,     1.0, 1.0,
            1.0, -1.0, 0.0,     1.0, 0.0,
            -1.0, -1.0, 0.0,    0.0, 0.0,
            -1.0, 1.0, 0.0,    0.0, 1.0
        ];

        #[rustfmt::skip]
        let indices = [
            0, 1, 3,
            1, 2, 3,
        ];

        let uniforms = [width as f32, height as f32, 0.0];

        let vertex_buffer = gfx
            .create_vertex_buffer()
            .with_info(&vertex_info)
            .with_data(&vertices)
            .build()
            .unwrap();

        let index_buffer = gfx
            .create_index_buffer()
            .with_data(&indices)
            .build()
            .unwrap();

        let uniform_buffer = gfx
            .create_uniform_buffer(0, "Locals")
            .with_data(&uniforms)
            .build()
            .unwrap();

        Self {
            render_texture,
            pipeline,
            value: 0.0,
            vertex_buffer,
            index_buffer,
            uniform_buffer,
        }
    }

    fn create_renderer(&mut self, gfx: &mut Graphics, delta: f32) -> Renderer {
        gfx.set_buffer_data(
            &self.uniform_buffer,
            &[
                WINDOW_WIDTH as f32,
                WINDOW_HEIGHT as f32,
                5.5 + self.value.sin(),
            ],
        );
        self.value += 0.3 * delta;

        let mut renderer = gfx.create_renderer();

        renderer.begin(None);
        renderer.set_pipeline(&self.pipeline);
        renderer.bind_texture(0, &self.render_texture);
        renderer.bind_buffers(&[
            &self.vertex_buffer,
            &self.index_buffer,
            &self.uniform_buffer,
        ]);
        renderer.draw(0, 6);
        renderer.end();

        renderer
    }
}
