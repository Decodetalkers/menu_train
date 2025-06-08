use godot::prelude::*;

mod player;
mod world;
mod main_scene;
mod splash;
mod settings;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}
