use std::cell::OnceCell;
use std::collections::HashMap;

use godot::classes::mesh;
use godot::classes::IStaticBody3D;
use godot::classes::Material;
use godot::classes::MeshInstance3D;
use godot::classes::StaticBody3D;
use godot::classes::SurfaceTool;
use godot::classes::WorkerThreadPool;
use godot::prelude::*;

use crate::settings::*;
use crate::world::MainWorld;

fn flat(chunk_position: Vector3i) -> HashMap<Vector3i, i32> {
    if chunk_position.y != -1 {
        return HashMap::new();
    }
    let mut data = HashMap::new();

    for x in 0..CHUNK_SIZE_I {
        for z in 0..CHUNK_SIZE_I {
            data.insert(Vector3i { x, y: 0, z }, 3);
        }
    }

    data
}

#[derive(GodotClass)]
#[class(base=StaticBody3D)]
pub struct Chunk {
    #[var]
    chunk_position: Vector3i,
    base: Base<StaticBody3D>,
    data: HashMap<Vector3i, i32>,
    world: OnceCell<Gd<MainWorld>>,
}

#[inline]
fn calculate_block_verts(Vector3i { x, y, z }: Vector3i) -> Vec<Vector3> {
    let x = x as f32;
    let y = y as f32;
    let z = z as f32;
    vec![
        Vector3::new(x, y, z),
        Vector3::new(x, y, z + 1.),
        Vector3::new(x, y + 1., z),
        Vector3::new(x, y + 1., z + 1.),
        Vector3::new(x + 1., y, z),
        Vector3::new(x + 1., y, z + 1.),
        Vector3::new(x + 1., y + 1., z),
        Vector3::new(x + 1., y + 1., z + 1.),
    ]
}

fn calculate_block_uvs(block_id: i32) -> Vec<Vector2> {
    let row = (block_id / TEXTURE_SHEET_WIDTH) as f32;
    let col = (block_id % TEXTURE_SHEET_WIDTH) as f32;
    vec![
        TEXTURE_TILE_SIZE * Vector2::new(col + 0.01, row + 0.01),
        TEXTURE_TILE_SIZE * Vector2::new(col + 0.01, row + 0.99),
        TEXTURE_TILE_SIZE * Vector2::new(col + 0.99, row + 0.01),
        TEXTURE_TILE_SIZE * Vector2::new(col + 0.99, row + 0.99),
    ]
}

impl Chunk {
    fn world(&self) -> &Gd<MainWorld> {
        self.world.get().unwrap()
    }
    fn world_mut(&mut self) -> &mut Gd<MainWorld> {
        self.world.get_mut().unwrap()
    }
}

#[godot_api]
impl Chunk {
    #[func]
    fn generate_chunk_mesh(&mut self) {
        if self.data.is_empty() {
            return;
        }
        let mut surface_tool = SurfaceTool::new_gd();
        surface_tool.begin(mesh::PrimitiveType::TRIANGLES);

        for block_position in self.data.keys() {
            let block_id = self.data.get(block_position).unwrap();
            self.draw_block_mesh(&surface_tool, *block_position, *block_id);
        }

        surface_tool.generate_normals();
        surface_tool.generate_tangents();
        surface_tool.index();

        let array_mesh = surface_tool.commit().unwrap();
        let mut mi = MeshInstance3D::new_alloc();
        mi.set_mesh(&array_mesh);

        let mut ma = Material::new_gd();
        ma.set_path("res://meterials/material.tres");
        mi.set_material_overlay(&ma);
        self.base_mut().add_child(&mi);
    }

    fn draw_block_mesh(&self, tool: &SurfaceTool, sub_position: Vector3i, id: i32) {
        let verts = calculate_block_verts(sub_position);
        let uvs = calculate_block_uvs(id);

        let top_uvs = calculate_block_uvs(0);
        let bottom_uvs = calculate_block_uvs(2);

        let other_block_position = sub_position + Vector3i::LEFT;

    }
}

#[godot_api]
impl IStaticBody3D for Chunk {
    fn init(base: Base<StaticBody3D>) -> Self {
        Self {
            base,
            chunk_position: Vector3i::ZERO,
            data: HashMap::new(),
            world: OnceCell::new(),
        }
    }
    fn ready(&mut self) {
        let world: Gd<MainWorld> = self.base().get_parent().unwrap().cast();
        self.world.set(world).unwrap();
        let mut transform = self.base().get_transform();
        transform.origin = (self.chunk_position * (CHUNK_SIZE as i32)).cast_float();
        self.base_mut().set_transform(transform);
        self.data = flat(self.chunk_position);
        let mut thread_pool = WorkerThreadPool::singleton();
        thread_pool.add_task(&Callable::from_object_method(
            &self.to_gd(),
            "generate_chunk_mesh",
        ));
    }
}
