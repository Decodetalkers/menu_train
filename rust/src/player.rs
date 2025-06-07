use godot::classes::AnimatedSprite2D;
use godot::classes::CharacterBody2D;
use godot::classes::ICharacterBody2D;
use godot::classes::Input;
use godot::obj::WithBaseField;
use godot::prelude::*;

#[derive(GodotClass)]
#[class(base=CharacterBody2D)]
struct Player {
    base: Base<CharacterBody2D>,
}
fn get_direction(input: &Input) -> Vector2 {
    input.get_vector("ui_left", "ui_right", "ui_up", "ui_down")
}

impl Player {
    fn animate_spite(&self) -> Gd<AnimatedSprite2D> {
        self.base().get_node_as("AnimatedSprite2D")
    }
}

#[godot_api]
impl Player {
    #[signal]
    fn door_opened();
}

#[godot_api]
impl ICharacterBody2D for Player {
    fn init(base: Base<CharacterBody2D>) -> Self {
        Self { base }
    }
    fn ready(&mut self) {}
    fn physics_process(&mut self, delta: f64) {
        let input = Input::singleton();
        let mut change = Vector2::new(0., 0.);
        if input.is_action_pressed("ui_left") {
            change += Vector2::LEFT;
        }
        if input.is_action_pressed("ui_right") {
            change += Vector2::RIGHT;
        }
        if input.is_action_pressed("ui_up") {
            change += Vector2::UP;
        }
        if input.is_action_pressed("ui_down") {
            change += Vector2::DOWN;
        }
        change = change * real::from_f64(delta) * 400.;
        let direction = get_direction(&input);
        let mut animate_spite = self.animate_spite();
        if direction.is_zero_approx() {
            animate_spite.set_animation("idle");
        } else {
            animate_spite.set_animation("run");
        }
        if direction.x < 0. {
            animate_spite.set_flip_h(true);
        } else {
            animate_spite.set_flip_h(false);
        }
        self.base_mut().move_and_collide(change);
    }
}
