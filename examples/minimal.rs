use std::f32::consts::TAU;

use bevy::prelude::*;
use bevy::window::CursorGrabMode;
use bevy_rapier3d::prelude::*;

use bevy_fps_controller::controller::*;

fn main() {
    App::new()
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 0.25,
        })
        .insert_resource(ClearColor(Color::rgb(0.752, 0.992, 0.984)))
        .insert_resource(RapierConfiguration::default())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                title: String::from("Minimal FPS Controller Example"),
                ..default()
            },
            ..default()
        }))
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        // .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(FpsControllerPlugin)
        .add_startup_system(setup)
        .add_system(manage_cursor)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 2000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(-38.0, 40.0, 34.0),
        ..default()
    });

    // Note that we have two entities for the player
    // One is a "logical" player that handles the physics computation and collision
    // The other is a "render" player that is what is displayed to the user
    // This distinction is useful for later on if you want to add multiplayer,
    // where often time these two ideas are not exactly synced up
    commands.spawn((
        Collider::capsule(Vec3::Y * 0.5, Vec3::Y * 1.5, 0.5),
        ActiveEvents::COLLISION_EVENTS,
        Velocity::zero(),
        RigidBody::Dynamic,
        Sleeping::disabled(),
        LockedAxes::ROTATION_LOCKED,
        AdditionalMassProperties::Mass(1.0),
        GravityScale(0.0),
        Ccd { enabled: true }, // Prevent clipping when going fast
        Transform::from_xyz(0.0, 3.0, 0.0),
        LogicalPlayer(0),
        FpsControllerInput {
            pitch: -TAU / 12.0,
            yaw: TAU * 5.0 / 8.0,
            ..default()
        },
        FpsController { ..default() }
    ));
    commands.spawn((
        Camera3dBundle::default(),
        RenderPlayer(0),
    ));

    // World
    commands.spawn_empty()
        .insert(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box {
                min_x: -20.0,
                max_x: 20.0,
                min_y: -0.25,
                max_y: 0.25,
                min_z: -20.0,
                max_z: 20.0,
            })),
            material: materials.add(StandardMaterial {
                base_color: Color::hex("E6EED6").unwrap(),
                ..default()
            }),
            transform: Transform::IDENTITY,
            ..default()
        })
        .insert(Collider::cuboid(20.0, 0.25, 20.0))
        .insert(RigidBody::Fixed)
        .insert(Transform::IDENTITY);

    commands.spawn_empty()
        .insert(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box {
                min_x: -1.0,
                max_x: 1.0,
                min_y: -1.0,
                max_y: 1.0,
                min_z: -1.0,
                max_z: 1.0,
            })),
            material: materials.add(StandardMaterial {
                base_color: Color::hex("DDE2C6").unwrap(),
                ..default()
            }),
            transform: Transform::IDENTITY,
            ..default()
        })
        .insert(Collider::cuboid(1.0, 1.0, 1.0))
        .insert(RigidBody::Fixed)
        .insert(Transform::from_xyz(4.0, 1.0, 4.0));
}

pub fn manage_cursor(
    mut windows: ResMut<Windows>,
    btn: Res<Input<MouseButton>>,
    key: Res<Input<KeyCode>>,
    mut controllers: Query<&mut FpsController>,
) {
    let window = windows.get_primary_mut().unwrap();
    if btn.just_pressed(MouseButton::Left) {
        window.set_cursor_grab_mode(CursorGrabMode::Locked);
        window.set_cursor_visibility(false);
        for mut controller in &mut controllers {
            controller.enable_input = true;
        }
    }
    if key.just_pressed(KeyCode::Escape) {
        window.set_cursor_grab_mode(CursorGrabMode::None);
        window.set_cursor_visibility(true);
        for mut controller in &mut controllers {
            controller.enable_input = false;
        }
    }
}
