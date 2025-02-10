use egui::{Painter, Pos2};

#[derive(Debug)]
pub struct Bezier(Pos2, Pos2, Pos2, Pos2);

impl Bezier {
    pub fn new(a: Pos2, b: Pos2, c: Pos2, d: Pos2) -> Self {
        Self(a, b, c, d)
    }
    pub fn eval(&self, t: f32) -> Pos2 {
        Pos2::ZERO +
            (
                (1.0 - t).powi(3) * self.0.to_vec2()
                + 3.0 * (1.0 - t).powi(2) * t * self.1.to_vec2()
                + 3.0 * (1.0 - t) * t.powi(2) * self.2.to_vec2()
                + t.powi(3) * self.3.to_vec2()
            )
    }
}

pub fn draw(painter: &mut Painter) {
    
}