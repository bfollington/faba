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

    // CRT effect parameters
    const float hardScan = -8.0;
    const float hardPix = -8.0;
    const float warpX = 0.1;
    const float warpY = 0.1;
    const float maskDark = 0.75;
    const float maskLight = 1.5;
    const float shadowMask = 3.0;
    const float brightBoost = 1.0;
    const float hardBloomPix = -1.5;
    const float hardBloomScan = -2.0;
    const float bloomAmount = 0.1;
    const float shape = 3.0;

    // Helper functions
    float ToLinear1(float c) {
        return c <= 0.04045 ? c / 12.92 : pow((c + 0.055) / 1.055, 2.4);
    }

    vec3 ToLinear(vec3 c) {
        return vec3(ToLinear1(c.r), ToLinear1(c.g), ToLinear1(c.b));
    }

    float ToSrgb1(float c) {
        return c < 0.0031308 ? c * 12.92 : 1.055 * pow(c, 0.41666) - 0.055;
    }

    vec3 ToSrgb(vec3 c) {
        return vec3(ToSrgb1(c.r), ToSrgb1(c.g), ToSrgb1(c.b));
    }

    vec3 Fetch(vec2 pos, vec2 off) {
        pos = (floor(pos * u_tex_size + off) + vec2(0.5, 0.5)) / u_tex_size;
        return ToLinear(brightBoost * texture(u_texture, pos).rgb);
    }

    vec2 Dist(vec2 pos) {
        pos = pos * u_tex_size;
        return -(pos - floor(pos) - vec2(0.5));
    }

    float Gaus(float pos, float scale) {
        return exp2(scale * pow(abs(pos), shape));
    }

    vec3 Horz3(vec2 pos, float off) {
        vec3 b = Fetch(pos, vec2(-1.0, off));
        vec3 c = Fetch(pos, vec2( 0.0, off));
        vec3 d = Fetch(pos, vec2( 1.0, off));
        float dst = Dist(pos).x;
        float scale = hardPix;
        float wb = Gaus(dst - 1.0, scale);
        float wc = Gaus(dst + 0.0, scale);
        float wd = Gaus(dst + 1.0, scale);
        return (b * wb + c * wc + d * wd) / (wb + wc + wd);
    }

    vec3 Horz5(vec2 pos, float off) {
        vec3 a = Fetch(pos, vec2(-2.0, off));
        vec3 b = Fetch(pos, vec2(-1.0, off));
        vec3 c = Fetch(pos, vec2( 0.0, off));
        vec3 d = Fetch(pos, vec2( 1.0, off));
        vec3 e = Fetch(pos, vec2( 2.0, off));
        float dst = Dist(pos).x;
        float scale = hardPix;
        float wa = Gaus(dst - 2.0, scale);
        float wb = Gaus(dst - 1.0, scale);
        float wc = Gaus(dst + 0.0, scale);
        float wd = Gaus(dst + 1.0, scale);
        float we = Gaus(dst + 2.0, scale);
        return (a * wa + b * wb + c * wc + d * wd + e * we) / (wa + wb + wc + wd + we);
    }

    float Scan(vec2 pos, float off) {
        float dst = Dist(pos).y;
        return Gaus(dst + off, hardScan);
    }

    vec3 Tri(vec2 pos) {
        vec3 a = Horz3(pos, -1.0);
        vec3 b = Horz5(pos,  0.0);
        vec3 c = Horz3(pos,  1.0);
        float wa = Scan(pos, -1.0);
        float wb = Scan(pos,  0.0);
        float wc = Scan(pos,  1.0);
        return a * wa + b * wb + c * wc;
    }

    vec2 Warp(vec2 pos) {
        // return pos;
        pos = pos * 2.0 - 1.0;
        pos *= vec2(1.0 + (pos.y * pos.y) * warpX, 1.0 + (pos.x * pos.x) * warpY);
        return (pos + 0.5) * 0.5 + vec2(0., 0.5);
    }

    vec3 Mask(vec2 pos) {
        pos.x += pos.y * 3.0;
        vec3 mask = vec3(maskDark, maskDark, maskDark);
        pos.x = fract(pos.x / 6.0);
        if (pos.x < 0.333) mask.r = maskLight;
        else if (pos.x < 0.666) mask.g = maskLight;
        else mask.b = maskLight;
        return mask;
    }

    const float simpleNoiseStrength = 0.005;
    const float perlinNoiseStrength = 0.01;
    const float noiseSpeed = 2.0;

    // Simple noise function
    float simpleNoise(vec2 p) {
        return fract(sin(dot(p.xy, vec2(12.9898,78.233))) * 43758.5453);
    }

    // Perlin noise functions
    vec4 permute(vec4 x) {
        return mod(((x*34.0)+1.0)*x, 289.0);
    }

    vec2 fade(vec2 t) {
        return t*t*t*(t*(t*6.0-15.0)+10.0);
    }

    float perlinNoise(vec2 P) {
        vec4 Pi = floor(P.xyxy) + vec4(0.0, 0.0, 1.0, 1.0);
        vec4 Pf = fract(P.xyxy) - vec4(0.0, 0.0, 1.0, 1.0);
        Pi = mod(Pi, 289.0);
        vec4 ix = Pi.xzxz;
        vec4 iy = Pi.yyww;
        vec4 fx = Pf.xzxz;
        vec4 fy = Pf.yyww;
        vec4 i = permute(permute(ix) + iy);
        vec4 gx = 2.0 * fract(i * 0.0243902439) - 1.0;
        vec4 gy = abs(gx) - 0.5;
        vec4 tx = floor(gx + 0.5);
        gx = gx - tx;
        vec2 g00 = vec2(gx.x,gy.x);
        vec2 g10 = vec2(gx.y,gy.y);
        vec2 g01 = vec2(gx.z,gy.z);
        vec2 g11 = vec2(gx.w,gy.w);
        vec4 norm = 1.79284291400159 - 0.85373472095314 * vec4(dot(g00, g00), dot(g01, g01), dot(g10, g10), dot(g11, g11));
        g00 *= norm.x;
        g01 *= norm.y;
        g10 *= norm.z;
        g11 *= norm.w;
        float n00 = dot(g00, vec2(fx.x, fy.x));
        float n10 = dot(g10, vec2(fx.y, fy.y));
        float n01 = dot(g01, vec2(fx.z, fy.z));
        float n11 = dot(g11, vec2(fx.w, fy.w));
        vec2 fade_xy = fade(Pf.xy);
        vec2 n_x = mix(vec2(n00, n01), vec2(n10, n11), fade_xy.x);
        float n_xy = mix(n_x.x, n_x.y, fade_xy.y);
        return 2.3 * n_xy;
    }

    // Apply both noise distortions
    vec2 applyNoiseDistortion(vec2 coord, float time) {
        // Simple noise distortion
        vec2 simpleNoise = vec2(simpleNoise(coord + time), simpleNoise(coord - time));
        coord += simpleNoise * simpleNoiseStrength;

        // Perlin noise displacement field
        float perlinValue = perlinNoise(coord * 5.0 + time);
        vec2 perlinDisplacement = vec2(cos(perlinValue), sin(perlinValue)) * perlinNoiseStrength;

        return coord + perlinDisplacement;
    }

    void main() {
        // Step 0: Apply noise distortion
        vec2 noisyCoord = applyNoiseDistortion(v_texcoord, u_value * noiseSpeed);

        // Step 1: Initial 2x scaling
        vec2 scaledCoord = (noisyCoord + 0.5) * 0.5;

        // Step 2: Apply CRT effects
        vec2 warpedCoord = Warp(scaledCoord);
        vec3 crtColor = Tri(warpedCoord);

        if (shadowMask > 0.0) {
            crtColor *= Mask(gl_FragCoord.xy * 1.000001);
        }

        // Step 3: Final color adjustment
        vec3 finalColor = ToSrgb(crtColor);

        outColor = vec4(finalColor, 1.0);
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
