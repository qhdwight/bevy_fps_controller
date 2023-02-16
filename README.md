[![Rust](https://github.com/qhdwight/bevy_fps_controller/actions/workflows/rust.yml/badge.svg)](https://github.com/qhdwight/bevy_fps_controller/actions/workflows/rust.yml)
[![crates.io](https://img.shields.io/crates/v/bevy_fps_controller)](https://crates.io/crates/bevy_fps_controller)

# Bevy FPS Controller

Inspired from Source engine movement, this plugin implements movement suitable for FPS games.

⚠️ Feedback requested! Still in early stages, feel free to make issues/PRs

### Features

* Air strafing and bunny hopping (hold down jump key)
* Crouching, sprinting
* Noclip mode
* Configurable settings

### Examples

See [main.rs](./examples/minimal.rs)

```bash
cargo run --example minimal
```

```rust
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use bevy_fps_controller::controller::*;

fn main() {
    App::new()
        ...
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(FpsControllerPlugin)
        ...
}

fn setup(...) {
    ...
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
            TransformBundle::from_transform(Transform::from_xyz(0.0, 3.0, 0.0)),
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
    ...
}
```

### Demo

Used by my other project: https://github.com/qhdwight/voxel-game-rs

https://user-images.githubusercontent.com/20666629/157115719-719a1e7b-a308-4239-919f-8daa9f2ef6e3.mp4
