use bevy::{
    prelude::*,
    window::WindowDescriptor,
};

use bevy_fps_controller::controller::*;
use bevy_rapier3d::prelude::*;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: String::from("Minimal FPS Controller Example"),
            ..default()
        })
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 0.25,
        })
        .insert_resource(RapierConfiguration::default())
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(FpsControllerPlugin)
        .run();
}
