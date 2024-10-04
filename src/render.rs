use notan::prelude::*;

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
        vec2 coord = v_texcoord / 2.0 + vec2(0., 0.5) - vec2(0.1 * (sin(u_value)), 0.1 * (cos(u_value)));
        vec4 tex_color = texture(u_texture, coord);
        outColor = tex_color;
    }
    "#
};

pub struct PostProcessTarget {
    pub render_texture: RenderTexture,
    pipeline: Pipeline,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    uniform_buffer: Buffer,
    value: f32,
}

impl PostProcessTarget {
    pub fn new(gfx: &mut Graphics, width: u32, height: u32) -> Self {
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

    pub fn create_renderer(&mut self, gfx: &mut Graphics, delta: f32) -> Renderer {
        gfx.set_buffer_data(
            &self.uniform_buffer,
            &[
                self.render_texture.width() as f32,
                self.render_texture.height() as f32,
                self.value,
            ],
        );
        self.value += delta;

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
