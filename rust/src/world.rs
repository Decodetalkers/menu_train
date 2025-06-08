use godot::classes::CharacterBody3D;
use godot::classes::INode3D;
use godot::classes::Node3D;

use godot::global::clampf;
use godot::prelude::*;

use crate::settings::*;
#[derive(GodotClass)]
#[class(base=Node3D)]
struct MainWorld {
    base: Base<Node3D>,
    player_chunk: Vector3i,
    deleting: bool,
    generating: bool,
}

impl MainWorld {
    fn player(&self) -> Gd<CharacterBody3D> {
        self.base().get_node_as("Player")
    }
}

#[godot_api]
impl INode3D for MainWorld {
    fn init(base: Base<Node3D>) -> Self {
        Self {
            base,
            player_chunk: Vector3i::ZERO,
            deleting: false,
            generating: false,
        }
    }
    fn process(&mut self, _delta: f64) {
        let player = self.player();

        let player_chunk_f: Vector3 = (player.get_transform().origin / CHUNK_SIZE).round();
        let mut player_chunk = Vector3i::new(
            player_chunk_f.x as i32,
            player_chunk_f.y as i32,
            player_chunk_f.z as i32,
        );

        if self.deleting || self.player_chunk != player_chunk {
            // TODO: delete away chunks
            self.generating = true;
        }
        if !self.generating {
            return;
        }
        let velocity = player.get_velocity();
        player_chunk.y += clampf(
            velocity.y as f64,
            -RENDER_DISTANCE as f64 / 4.,
            RENDER_DISTANCE as f64 / 4.,
        )
        .round() as i32;
    }
}
