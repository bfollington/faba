use notan::math::Rect;

pub trait Vertices {
    fn vertices(&self) -> Vec<(f32, f32)>;
}

impl Vertices for Rect {
    fn vertices(&self) -> Vec<(f32, f32)> {
        vec![
            (self.x, self.y),
            (self.x + self.width, self.y),
            (self.x + self.width, self.y + self.height),
            (self.x, self.y + self.height),
        ]
    }
}

pub trait ToVec2 {
    fn to_vec2(&self) -> notan::math::Vec2;
}

impl ToVec2 for (f32, f32) {
    fn to_vec2(&self) -> notan::math::Vec2 {
        notan::math::Vec2::new(self.0, self.1)
    }
}

pub trait ToTuple {
    fn to_tuple(&self) -> (f32, f32);
}

impl ToTuple for notan::math::Vec2 {
    fn to_tuple(&self) -> (f32, f32) {
        (self.x, self.y)
    }
}
