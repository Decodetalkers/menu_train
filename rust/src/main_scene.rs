use godot::classes::Control;
use godot::classes::IControl;
use godot::classes::TextureButton;

use godot::prelude::*;

#[derive(GodotClass)]
#[class(base=Control)]
struct MainSene {
    base: Base<Control>,
}

impl MainSene {
    fn start_button(&self) -> Gd<TextureButton> {
        self.base()
            .get_node_as("VBoxContainer/BoxHolder/MainButtons/Start")
    }
}

#[godot_api]
impl IControl for MainSene {
    fn init(base: Base<Control>) -> Self {
        Self { base }
    }
    fn ready(&mut self) {
        let button = self.start_button();
        button.signals().pressed().connect_other(self, |scene| {
            let mut tree = scene.base().get_tree().unwrap();
            tree.change_scene_to_packed(&load::<PackedScene>("res://world.tscn"));
        });
    }
}
