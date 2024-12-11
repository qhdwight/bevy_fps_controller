[![Rust](https://github.com/qhdwight/bevy_fps_controller/actions/workflows/rust.yml/badge.svg)](https://github.com/qhdwight/bevy_fps_controller/actions/workflows/rust.yml)
[![crates.io](https://img.shields.io/crates/v/bevy_fps_controller)](https://crates.io/crates/bevy_fps_controller)

# Bevy FPS Controller

Inspired from Source engine movement, this plugin implements movement suitable for FPS games.

Feel free to make issues/PRs!

### Features

* Air strafing and bunny hopping (hold down jump key)
* Support for sloped ground
* Crouching (prevents falling off ledges), sprinting
* Noclip mode
* Configurable settings

### Examples

See [main.rs](./examples/minimal.rs)

```bash
cargo run --release --example minimal
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
        .add_startup_system(setup)
        ...
}

fn setup(mut commands: Commands, ...) {
    ...
    let logical_entity = commands
        .spawn((
            Collider::capsule_y(1.0, 0.5),
            Friction {
                coefficient: 0.0,
                combine_rule: CoefficientCombineRule::Min,
            },
            Restitution {
                coefficient: 0.0,
                combine_rule: CoefficientCombineRule::Min,
            },
            ActiveEvents::COLLISION_EVENTS,
            Velocity::zero(),
            RigidBody::Dynamic,
            Sleeping::disa  bled(),
            LockedAxes::ROTATION_LOCKED,
            AdditionalMassProperties::Mass(1.0),
            GravityScale(0.0),
            Ccd { enabled: true }, // Prevent clipping when going fast
            TransformBundle::from_transform(Transform::from_xyz(0.0, 3.0, 0.0)),
            LogicalPlayer,
            FpsControllerInput {
                pitch: -TAU / 12.0,
                yaw: TAU * 5.0 / 8.0,
                ..default()
            },
            FpsController { ..default() }
        ))
        .insert(CameraConfig {
            height_offset: -0.5,
        })
        .id();

    commands.spawn((
        Camera3dBundle::default(),
        RenderPlayer { logical_entity },
    ));
    ...
}
```

### Demo

https://user-images.githubusercontent.com/20666629/221995601-2ec352fe-a8b0-4f8c-9a81-beaf898b2b41.mp4

Used by my other project: https://github.com/qhdwight/voxel-game-rs
