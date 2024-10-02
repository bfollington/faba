use notan::draw::*;
use notan::math::Vec2;
use notan::prelude::*;
use rapier2d::prelude::*;

const MOVE_SPEED: f32 = 50.0;
const JUMP_FORCE: f32 = -150.0;

pub struct Player {
    pub body_handle: RigidBodyHandle,
    pub collider_handle: ColliderHandle,
    pub size: Vec2,
    pub is_grounded: bool,
    pub jump_buffer: f32,
    pub coyote_time: f32,
}

impl Player {
    pub fn new(
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        bodies: &mut RigidBodySet,
        colliders: &mut ColliderSet,
    ) -> Self {
        let rigid_body = RigidBodyBuilder::dynamic()
            .translation(vector![x, y])
            .lock_rotations()
            .build();
        let body_handle = bodies.insert(rigid_body);
        let radius = width / 2.0;
        let half_height = (height - width) / 2.0;
        let collider = ColliderBuilder::capsule_y(half_height, radius)
            .friction(0.0)
            .build();
        let collider_handle = colliders.insert_with_parent(collider, body_handle, bodies);

        Self {
            body_handle,
            collider_handle,
            size: Vec2::new(width, height),
            is_grounded: false,
            jump_buffer: 0.0,
            coyote_time: 0.0,
        }
    }

    pub fn update(
        &mut self,
        bodies: &RigidBodySet,
        colliders: &ColliderSet,
        query: &QueryPipeline,
    ) {
        if let Some(body) = bodies.get(self.body_handle) {
            let velocity = body.linvel();
            let position = body.translation();

            let (rays, filter) = self.generate_ground_check_rays(position);
            self.is_grounded = self.check_grounded(&rays, bodies, colliders, query, filter);
        }
    }

    fn generate_ground_check_rays(&self, position: &Vector<Real>) -> (Vec<Ray>, QueryFilter) {
        let ray_pos = point![position.x, position.y + self.size.y / 2.0];
        let ray_dir = vector![0.0, 1.0];
        let ray_length = 2.0;
        let filter = QueryFilter::default()
            .exclude_rigid_body(self.body_handle)
            .exclude_collider(self.collider_handle);

        let ray_offsets = [-self.size.x / 3.0, 0.0, self.size.x / 3.0];
        let rays = ray_offsets
            .iter()
            .map(|offset| Ray {
                dir: ray_dir,
                origin: point![ray_pos.x + offset, ray_pos.y],
            })
            .collect();

        (rays, filter)
    }

    fn check_grounded(
        &self,
        rays: &[Ray],
        bodies: &RigidBodySet,
        colliders: &ColliderSet,
        query: &QueryPipeline,
        filter: QueryFilter,
    ) -> bool {
        for ray in rays {
            if query
                .cast_ray(bodies, colliders, ray, 2.0, true, filter)
                .is_some()
            {
                return true;
            }
        }
        false
    }

    pub fn debug_render_ground_check_rays(&self, draw: &mut Draw, bodies: &RigidBodySet) {
        if let Some(body) = bodies.get(self.body_handle) {
            let position = body.translation();
            let (rays, _) = self.generate_ground_check_rays(position);

            for ray in rays {
                let start = (ray.origin.x, ray.origin.y);
                let end = (
                    ray.origin.x + ray.dir.x * 2.0,
                    ray.origin.y + ray.dir.y * 2.0,
                );
                draw.line(start, end).color(Color::GREEN);
            }
        }
    }

    pub fn set_movement(&self, move_left: bool, move_right: bool, bodies: &mut RigidBodySet) {
        let mut move_dir = 0.0;
        if move_left {
            move_dir -= 1.0;
        }
        if move_right {
            move_dir += 1.0;
        }

        if let Some(body) = bodies.get_mut(self.body_handle) {
            let mut velocity = *body.linvel();
            velocity.x = move_dir * MOVE_SPEED;

            // Apply slope adjustment
            if self.is_grounded {
                let slope_angle = velocity.y.atan2(velocity.x);
                velocity.y = velocity.x * slope_angle.sin();
            }

            body.set_linvel(velocity, true);
        }
    }

    pub fn jump(&mut self, bodies: &mut RigidBodySet) {
        if self.is_grounded || self.coyote_time > 0.0 {
            if let Some(body) = bodies.get_mut(self.body_handle) {
                let mut velocity = *body.linvel();
                velocity.y = JUMP_FORCE;
                body.set_linvel(velocity, true);
            }
        } else {
            self.jump_buffer = 0.1; // Set jump buffer for 100ms
        }
    }

    pub fn position(&self, bodies: &RigidBodySet) -> Vec2 {
        if let Some(body) = bodies.get(self.body_handle) {
            let position = body.translation();
            Vec2::new(position.x, position.y)
        } else {
            Vec2::ZERO
        }
    }

    pub fn render(&self, draw: &mut Draw, bodies: &RigidBodySet) {
        if let Some(body) = bodies.get(self.body_handle) {
            let position = body.translation();
            draw.rect(
                (
                    position.x - self.size.x / 2.0,
                    position.y - self.size.y / 2.0,
                ),
                (self.size.x, self.size.y),
            )
            .color(Color::BLUE);
        }
    }

    pub fn debug_render(&self, draw: &mut Draw, font: &Font, bodies: &RigidBodySet) {
        if let Some(body) = bodies.get(self.body_handle) {
            let position = body.translation();
            let velocity = body.linvel();

            // Draw velocity vector
            let vel_end = Vec2::new(position.x + velocity.x / 5.0, position.y + velocity.y / 5.0);
            draw.line((position.x, position.y), (vel_end.x, vel_end.y))
                .color(Color::RED);

            // Draw grounded indicator
            let grounded_text = if self.is_grounded {
                "Grounded"
            } else {
                "In Air"
            };
            draw.text(font, grounded_text)
                .position(position.x, position.y - self.size.y / 2.0 - 20.0)
                .size(16.0)
                .color(Color::WHITE);

            // Draw debug info
            let debug_text = format!(
                "Jump Buffer: {:.2}\nCoyote Time: {:.2}\nVelocity: ({:.2}, {:.2})",
                self.jump_buffer, self.coyote_time, velocity.x, velocity.y
            );
            draw.text(font, &debug_text)
                .position(10.0, 10.0)
                .size(16.0)
                .color(Color::WHITE);

            self.debug_render_ground_check_rays(draw, bodies);
        }
    }
}
