use godot::classes::CharacterBody3D;
use godot::classes::INode3D;
use godot::classes::Node;
use godot::classes::Node3D;

use godot::global::clampf;
use godot::prelude::*;

use crate::chunk::Chunk;
use crate::settings::*;
use std::collections::HashMap;

#[derive(GodotClass)]
#[class(base=Node3D)]
pub struct MainWorld {
    base: Base<Node3D>,
    player_chunk: Vector3i,
    deleting: bool,
    generating: bool,
    effective_render_distance: i32,
    chunks: HashMap<Vector3i, Gd<Chunk>>,
}

impl MainWorld {
    fn player(&self) -> Gd<CharacterBody3D> {
        self.base().get_node_as("Player")
    }
    fn the_world(&self) -> Gd<Node> {
        self.base().get_node_as("the_world")
    }
}

#[godot_api]
impl MainWorld {
    #[func]
    fn get_block_global_position(block_global_position: Vector3i) {

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
            effective_render_distance: 0,
            chunks: HashMap::new(),
        }
    }
    fn process(&mut self, _delta: f64) {
        let player = self.player();

        let mut player_chunk: Vector3i = (player.get_transform().origin / CHUNK_SIZE)
            .round()
            .cast_int();

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

        for x in player_chunk.x - self.effective_render_distance
            ..player_chunk.x + self.effective_render_distance
        {
            for y in player_chunk.y - self.effective_render_distance
                ..player_chunk.y + self.effective_render_distance
            {
                for z in player_chunk.z - self.effective_render_distance
                    ..player_chunk.z + self.effective_render_distance
                {
                    let chunk_position = Vector3i::new(x, y, z);
                    if player_chunk
                        .cast_float()
                        .distance_to(chunk_position.cast_float())
                        > RENDER_DISTANCE
                    {
                        continue;
                    }
                    if self.chunks.contains_key(&chunk_position) {
                        continue;
                    }
                    let mut chunk = Chunk::new_alloc();
                    chunk.bind_mut().set_chunk_position(chunk_position);
                    self.chunks.insert(chunk_position, chunk.clone());
                    self.the_world().add_child(&chunk);
                    return;
                }
            }
        }

        if self.effective_render_distance < RENDER_DISTANCE as i32 {
            self.effective_render_distance += 1;
        } else {
            self.generating = false
        }
    }
}
