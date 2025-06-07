use godot::classes::Node2D;

use godot::classes::CanvasLayer;
use godot::prelude::*;

#[derive(GodotClass)]
#[class(base=Node2D)]
struct MainSene {
    base: Base<Node2D>,
}

impl MainSene {
    fn text_box(&self) -> Gd<CanvasLayer> {
        self.base().get_node_as("TextBox")
    }
}

#[godot_api]
impl INode2D for MainSene {
    fn init(base: Base<Node2D>) -> Self {
        Self { base }
    }
    fn ready(&mut self) {
        self.text_box().hide();
    }
}
