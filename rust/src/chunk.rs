use godot::builtin::Dictionary;
use godot::classes::mesh;
use godot::classes::IStaticBody3D;
use godot::classes::Material;
use godot::classes::MeshInstance3D;
use godot::classes::StaticBody3D;
use godot::classes::SurfaceTool;
use godot::classes::WorkerThreadPool;
use godot::prelude::*;
use std::cell::OnceCell;

use crate::settings::*;
use crate::world::MainWorld;

fn flat(chunk_position: Vector3i) -> Dictionary {
    if chunk_position.y != -1 {
        return Dictionary::new();
    }
    let mut data = Dictionary::new();

    for x in 0..CHUNK_SIZE_I {
        for z in 0..CHUNK_SIZE_I {
            data.insert(Vector3i { x, y: 0, z }, 3).unwrap();
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
    #[var]
    data: Dictionary,
    world: OnceCell<Gd<MainWorld>>,
    task_id: i64,
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
    #[allow(unused)]
    fn world_mut(&mut self) -> &mut Gd<MainWorld> {
        self.world.get_mut().unwrap()
    }
}

#[inline]
fn is_block_transparent(block_id: i32) -> bool {
    block_id == 0 || (block_id > 25 && block_id < 30)
}

fn draw_block_face(surface_tool: &mut SurfaceTool, verts: &Vec<Vector3>, uvs: &Vec<Vector2>) {
    surface_tool.set_uv(uvs[1]);
    surface_tool.add_vertex(verts[1]);
    surface_tool.set_uv(uvs[2]);
    surface_tool.add_vertex(verts[2]);
    surface_tool.set_uv(uvs[3]);
    surface_tool.add_vertex(verts[3]);

    surface_tool.set_uv(uvs[2]);
    surface_tool.add_vertex(verts[2]);
    surface_tool.set_uv(uvs[1]);
    surface_tool.add_vertex(verts[1]);
    surface_tool.set_uv(uvs[0]);
    surface_tool.add_vertex(verts[0]);
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

        for block_position in self.data.keys_shared() {
            let block_id = self.data.get(block_position.clone()).unwrap();
            self.draw_block_mesh(
                &mut surface_tool,
                block_position.clone().try_to().unwrap(),
                block_id.try_to().unwrap(),
            );
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

    fn draw_block_mesh(
        &self,
        surface_tool: &mut SurfaceTool,
        block_sub_position: Vector3i,
        block_id: i32,
    ) {
        let verts = calculate_block_verts(block_sub_position);
        let uvs = calculate_block_uvs(block_id);

        let world = self.world().bind();

        let mut other_block_position = block_sub_position + Vector3i::LEFT;
        let mut other_block_id = 0;
        if other_block_position.x == -1 {
            other_block_id = world.get_block_global_position(other_block_position);
        } else if self.data.contains_key(other_block_position.clone()) {
            other_block_id = self
                .data
                .get(other_block_position.clone())
                .unwrap()
                .try_to()
                .unwrap();
        }

        if block_id != other_block_id && is_block_transparent(other_block_id) {
            draw_block_face(
                surface_tool,
                &vec![verts[2], verts[0], verts[3], verts[1]],
                &uvs,
            );
        }

        other_block_position = block_sub_position * Vector3i::RIGHT;
        other_block_id = 0;
        if other_block_position.x == CHUNK_SIZE_I {
            other_block_id = world.get_block_global_position(other_block_position);
        } else if self.data.contains_key(other_block_position.clone()) {
            other_block_id = self
                .data
                .get(other_block_position.clone())
                .unwrap()
                .try_to()
                .unwrap();
        }

        if block_id != other_block_id && is_block_transparent(other_block_id) {
            draw_block_face(
                surface_tool,
                &vec![verts[7], verts[5], verts[6], verts[4]],
                &uvs,
            );
        }

        other_block_position = block_sub_position * Vector3i::FORWARD;
        other_block_id = 0;
        if other_block_position.z == -1 {
            other_block_id = world.get_block_global_position(other_block_position);
        } else if self.data.contains_key(other_block_position.clone()) {
            other_block_id = self
                .data
                .get(other_block_position.clone())
                .unwrap()
                .try_to()
                .unwrap();
        }

        if block_id != other_block_id && is_block_transparent(other_block_id) {
            draw_block_face(
                surface_tool,
                &vec![verts[6], verts[4], verts[2], verts[0]],
                &uvs,
            );
        }

        other_block_position = block_sub_position * Vector3i::BACK;
        other_block_id = 0;
        if other_block_position.z == CHUNK_SIZE_I {
            other_block_id = world.get_block_global_position(other_block_position);
        } else if self.data.contains_key(other_block_position.clone()) {
            other_block_id = self
                .data
                .get(other_block_position.clone())
                .unwrap()
                .try_to()
                .unwrap();
        }

        if block_id != other_block_id && is_block_transparent(other_block_id) {
            draw_block_face(
                surface_tool,
                &vec![verts[3], verts[1], verts[7], verts[5]],
                &uvs,
            );
        }

        other_block_position = block_sub_position * Vector3i::DOWN;
        other_block_id = 0;
        if other_block_position.y == -1 {
            other_block_id = world.get_block_global_position(other_block_position);
        } else if self.data.contains_key(other_block_position.clone()) {
            other_block_id = self
                .data
                .get(other_block_position.clone())
                .unwrap()
                .try_to()
                .unwrap();
        }

        if block_id != other_block_id && is_block_transparent(other_block_id) {
            draw_block_face(
                surface_tool,
                &vec![verts[4], verts[5], verts[0], verts[1]],
                &uvs,
            );
        }

        other_block_position = block_sub_position * Vector3i::UP;
        other_block_id = 0;
        if other_block_position.y == CHUNK_SIZE_I {
            other_block_id = world.get_block_global_position(other_block_position);
        } else if self.data.contains_key(other_block_position.clone()) {
            other_block_id = self
                .data
                .get(other_block_position.clone())
                .unwrap()
                .try_to()
                .unwrap();
        }

        if block_id != other_block_id && is_block_transparent(other_block_id) {
            draw_block_face(
                surface_tool,
                &vec![verts[2], verts[3], verts[6], verts[7]],
                &uvs,
            );
        }
    }
}

#[godot_api]
impl IStaticBody3D for Chunk {
    fn init(base: Base<StaticBody3D>) -> Self {
        Self {
            base,
            chunk_position: Vector3i::ZERO,
            data: Dictionary::new(),
            world: OnceCell::new(),
            task_id: -1,
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
        let task_id = thread_pool.add_task(&Callable::from_object_method(
            &self.to_gd(),
            "generate_chunk_mesh",
        ));
        self.task_id = task_id;
    }
}
