use notan::{
    app::Color,
    draw::{Draw, DrawShapes},
    math::Vec2,
};

pub struct Bullet {
    pub position: Vec2,
    pub velocity: Vec2,
    pub life: f32,
}

pub struct Gun {
    pub angle: f32,
    pub aim_line_length: f32,
    pub shoot_radius: f32,
    pub bullets: Vec<Bullet>,
}

impl Gun {
    pub fn new() -> Self {
        Gun {
            angle: 0.0,
            aim_line_length: 50.0,
            shoot_radius: 5.0,
            bullets: Vec::new(),
        }
    }

    pub fn shoot(&mut self, player_position: (f32, f32)) {
        let x = player_position.0 + self.angle.cos() * self.shoot_radius;
        let y = player_position.1 + self.angle.sin() * self.shoot_radius;

        self.bullets.push(Bullet {
            position: Vec2::new(x, y),
            velocity: Vec2::new(self.angle.cos() * 500.0, self.angle.sin() * 500.0),
            life: 1.0,
        });
    }

    pub fn update(&mut self, player_position: (f32, f32), mouse_position: (f32, f32), dt: f32) {
        let dx = mouse_position.0 - player_position.0;
        let dy = mouse_position.1 - player_position.1;

        self.angle = dy.atan2(dx);
        self.aim_line_length = (dx * dx + dy * dy).sqrt();

        for bullet in &mut self.bullets {
            bullet.position += bullet.velocity * dt;
        }

        self.bullets.retain_mut(|bullet| {
            bullet.life -= dt;
            bullet.life > 0.0
        });
    }

    pub fn draw(&self, draw: &mut Draw, player_position: (f32, f32)) {
        let x = player_position.0 + self.angle.cos() * self.aim_line_length;
        let y = player_position.1 + self.angle.sin() * self.aim_line_length;

        draw.line((player_position.0, player_position.1), (x, y))
            .width(2.0)
            .color(Color::YELLOW)
            .alpha(0.2);

        for bullet in &self.bullets {
            draw.circle(5.0)
                .position(bullet.position.x, bullet.position.y)
                .color(Color::RED);
        }
    }
}
