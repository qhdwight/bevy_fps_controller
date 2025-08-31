use std::f32::consts::TAU;

use avian3d::prelude::*;
use bevy::{
    gltf::{Gltf, GltfMesh, GltfNode},
    math::Vec3Swizzles,
    prelude::*,
    render::camera::Exposure,
    window::CursorGrabMode,
};

use bevy_fps_controller::controller::*;

const SPAWN_POINT: Vec3 = Vec3::new(0.0, 1.625, 0.0);

fn main() {
    App::new()
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 10000.0,
            affects_lightmapped_meshes: true,
        })
        .insert_resource(ClearColor(Color::linear_rgb(0.83, 0.96, 0.96)))
        .add_plugins(DefaultPlugins)
        .add_plugins(PhysicsPlugins::default())
        // .add_plugins(PhysicsDebugPlugin::default())
        .add_plugins(FpsControllerPlugin)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (manage_cursor, scene_colliders, display_text, respawn),
        )
        .run();
}

fn setup(mut commands: Commands, mut window: Query<&mut Window>, assets: Res<AssetServer>) {
    let mut window = window.single_mut().unwrap();
    window.title = String::from("Minimal FPS Controller Example");

    commands.spawn((
        DirectionalLight {
            illuminance: light_consts::lux::FULL_DAYLIGHT,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 7.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Note that we have two entities for the player
    // One is a "logical" player that handles the physics computation and collision
    // The other is a "render" player that is what is displayed to the user
    // This distinction is useful for later on if you want to add multiplayer,
    // where often time these two ideas are not exactly synced up
    let height = 3.0;
    let logical_entity = commands
        .spawn((
            Collider::cylinder(0.5, height),
            // A capsule can be used but is NOT recommended
            // If you use it, you have to make sure each segment point is
            // equidistant from the translation of the player transform
            // Collider::capsule(0.5, height),
            Friction {
                dynamic_coefficient: 0.0,
                static_coefficient: 0.0,
                combine_rule: CoefficientCombine::Min,
            },
            Restitution {
                coefficient: 0.0,
                combine_rule: CoefficientCombine::Min,
            },
            LinearVelocity::ZERO,
            RigidBody::Dynamic,
            Sleeping,
            LockedAxes::ROTATION_LOCKED,
            Mass(1.0),
            GravityScale(0.0),
            Transform::from_translation(SPAWN_POINT),
            LogicalPlayer,
            FpsControllerInput {
                pitch: -TAU / 12.0,
                yaw: TAU * 5.0 / 8.0,
                ..default()
            },
            FpsController {
                air_acceleration: 80.0,
                ..default()
            },
        ))
        .insert(CameraConfig {
            height_offset: -0.5,
        })
        .id();

    commands.spawn((
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: TAU / 5.0,
            ..default()
        }),
        Exposure::SUNLIGHT,
        RenderPlayer { logical_entity },
    ));

    commands.insert_resource(MainScene {
        handle: assets.load("playground.glb"),
        is_loaded: false,
    });

    commands.spawn((
        Text(String::from("")),
        TextFont {
            font: assets.load("fira_mono.ttf"),
            font_size: 24.0,
            ..default()
        },
        TextColor(Color::BLACK),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(5.0),
            left: Val::Px(5.0),
            ..default()
        },
    ));
}

fn respawn(mut query: Query<(&mut Transform, &mut LinearVelocity)>) {
    for (mut transform, mut velocity) in &mut query {
        if transform.translation.y > -50.0 {
            continue;
        }

        velocity.0 = Vec3::ZERO;
        transform.translation = SPAWN_POINT;
    }
}

#[derive(Resource)]
struct MainScene {
    handle: Handle<Gltf>,
    is_loaded: bool,
}

fn scene_colliders(
    mut commands: Commands,
    mut main_scene: ResMut<MainScene>,
    gltf_assets: Res<Assets<Gltf>>,
    gltf_mesh_assets: Res<Assets<GltfMesh>>,
    gltf_node_assets: Res<Assets<GltfNode>>,
    mesh_assets: Res<Assets<Mesh>>,
) {
    if main_scene.is_loaded {
        return;
    }

    let gltf = gltf_assets.get(&main_scene.handle);

    if let Some(gltf) = gltf {
        let scene = gltf.scenes.first().unwrap().clone();
        commands.spawn(SceneRoot(scene));
        for node in &gltf.nodes {
            let node = gltf_node_assets.get(node).unwrap();
            if let Some(gltf_mesh) = node.mesh.clone() {
                let gltf_mesh = gltf_mesh_assets.get(&gltf_mesh).unwrap();
                for mesh_primitive in &gltf_mesh.primitives {
                    let mesh = mesh_assets.get(&mesh_primitive.mesh).unwrap();
                    commands.spawn((
                        Collider::trimesh_from_mesh(mesh).unwrap(),
                        RigidBody::Static,
                        node.transform,
                    ));
                }
            }
        }
        main_scene.is_loaded = true;
    }
}

fn manage_cursor(
    btn: Res<ButtonInput<MouseButton>>,
    key: Res<ButtonInput<KeyCode>>,
    mut window_query: Query<&mut Window>,
    mut controller_query: Query<&mut FpsController>,
) {
    for mut window in &mut window_query {
        if btn.just_pressed(MouseButton::Left) {
            window.cursor_options.grab_mode = CursorGrabMode::Locked;
            window.cursor_options.visible = false;
            for mut controller in &mut controller_query {
                controller.enable_input = true;
            }
        }
        if key.just_pressed(KeyCode::Escape) {
            window.cursor_options.grab_mode = CursorGrabMode::None;
            window.cursor_options.visible = true;
            for mut controller in &mut controller_query {
                controller.enable_input = false;
            }
        }
    }
}

fn display_text(
    mut controller_query: Query<(&Transform, &LinearVelocity), With<LogicalPlayer>>,
    mut text_query: Query<&mut Text>,
) {
    for (transform, velocity) in &mut controller_query {
        for mut text in &mut text_query {
            text.0 = format!(
                "vel: {:.2}, {:.2}, {:.2}\npos: {:.2}, {:.2}, {:.2}\nspd: {:.2}",
                velocity.0.x,
                velocity.0.y,
                velocity.0.z,
                transform.translation.x,
                transform.translation.y,
                transform.translation.z,
                velocity.0.xz().length()
            );
        }
    }
}
