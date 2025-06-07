use godot::classes::{Control, IControl};

use godot::prelude::*;

#[derive(GodotClass)]
#[class(base=Control)]
struct SplashController {
    base: Base<Control>,
    time: f64,
}

#[godot_api]
impl IControl for SplashController {
    fn init(base: Base<Control>) -> Self {
        Self { base, time: 0. }
    }
    fn process(&mut self, delta: f64) {
        self.time += delta;
        let scale = real::from_f64(1. - ((self.time * 4.).sin() / 4.).abs());
        self.base_mut().set_scale(Vector2::ONE * scale);
    }
}
