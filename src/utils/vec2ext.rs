use egui::Vec2;

pub trait Vec2Ext {
    fn rotate(self, angle: f32) -> Self;
}

impl Vec2Ext for Vec2 {
    fn rotate(self, angle: f32) -> Self {
        Vec2::new(
            self.x * angle.cos() - self.y * angle.sin(),
            self.x * angle.sin() + self.y * angle.cos(),
        )
    }
}
