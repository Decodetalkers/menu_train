use godot::prelude::*;

mod player;
mod main_scene;
mod splash;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}
