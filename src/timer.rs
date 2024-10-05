pub struct Timer {
    pub time: f32,
    pub duration: f32,
}

impl Timer {
    pub fn new(duration: f32) -> Self {
        Timer {
            time: 0.0,
            duration,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.time += dt;
    }

    pub fn is_finished(&self) -> bool {
        self.time >= self.duration
    }

    pub fn reset(&mut self) {
        self.time = 0.0;
    }

    pub fn progress(&self) -> f32 {
        (self.time / self.duration).clamp(0.0, 1.0)
    }
}
