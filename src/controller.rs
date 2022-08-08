use std::f32::consts::*;

use bevy::input::mouse::MouseMotion;
use bevy::{math::Vec3Swizzles, prelude::*};
use bevy_rapier3d::prelude::*;

pub struct FpsControllerPlugin;

impl Plugin for FpsControllerPlugin {
    fn build(&self, app: &mut App) {
        // TODO: these need to be sequential (exclusive system set)
        app.add_system(fps_controller_input)
            .add_system(fps_controller_look)
            .add_system(fps_controller_move)
            .add_system(fps_controller_render);
    }
}

pub enum MoveMode {
    Noclip,
    Ground,
}

#[derive(Component)]
pub struct LogicalPlayer(pub u8);

#[derive(Component)]
pub struct RenderPlayer(pub u8);

#[derive(Component, Default)]
pub struct FpsControllerInput {
    pub fly: bool,
    pub sprint: bool,
    pub jump: bool,
    pub crouch: bool,
    pub pitch: f32,
    pub yaw: f32,
    pub movement: Vec3,
}

#[derive(Component)]
pub struct FpsController {
    pub move_mode: MoveMode,
    pub gravity: f32,
    pub walk_speed: f32,
    pub run_speed: f32,
    pub forward_speed: f32,
    pub side_speed: f32,
    pub air_speed_cap: f32,
    pub air_acceleration: f32,
    pub max_air_speed: f32,
    pub accel: f32,
    pub friction: f32,
    pub friction_cutoff: f32,
    pub jump_speed: f32,
    pub fly_speed: f32,
    pub fast_fly_speed: f32,
    pub fly_friction: f32,
    pub pitch: f32,
    pub yaw: f32,
    pub velocity: Vec3,
    pub ground_tick: u8,
    pub stop_speed: f32,
    pub sensitivity: f32,
    pub enable_input: bool,
    pub key_forward: KeyCode,
    pub key_back: KeyCode,
    pub key_left: KeyCode,
    pub key_right: KeyCode,
    pub key_up: KeyCode,
    pub key_down: KeyCode,
    pub key_sprint: KeyCode,
    pub key_jump: KeyCode,
    pub key_fly: KeyCode,
    pub key_crouch: KeyCode,
}

impl Default for FpsController {
    fn default() -> Self {
        Self {
            move_mode: MoveMode::Ground,
            fly_speed: 10.0,
            fast_fly_speed: 30.0,
            gravity: 23.0,
            walk_speed: 10.0,
            run_speed: 30.0,
            forward_speed: 30.0,
            side_speed: 30.0,
            air_speed_cap: 2.0,
            air_acceleration: 20.0,
            max_air_speed: 8.0,
            accel: 10.0,
            friction: 10.0,
            friction_cutoff: 0.1,
            fly_friction: 0.5,
            pitch: 0.0,
            yaw: 0.0,
            velocity: Vec3::ZERO,
            ground_tick: 0,
            stop_speed: 1.0,
            jump_speed: 8.5,
            enable_input: true,
            key_forward: KeyCode::W,
            key_back: KeyCode::S,
            key_left: KeyCode::A,
            key_right: KeyCode::D,
            key_up: KeyCode::Q,
            key_down: KeyCode::E,
            key_sprint: KeyCode::LShift,
            key_jump: KeyCode::Space,
            key_fly: KeyCode::F,
            key_crouch: KeyCode::LControl,
            sensitivity: 0.001,
        }
    }
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝

const ANGLE_EPSILON: f32 = 0.001953125;

pub fn fps_controller_input(
    key_input: Res<Input<KeyCode>>,
    mut windows: ResMut<Windows>,
    mut mouse_events: EventReader<MouseMotion>,
    mut query: Query<(&FpsController, &mut FpsControllerInput)>,
) {
    for (controller, mut input) in query.iter_mut() {
        if !controller.enable_input {
            continue;
        }
        let window = windows.get_primary_mut().unwrap();
        if window.is_focused() {
            let mut mouse_delta = Vec2::ZERO;
            for mouse_event in mouse_events.iter() {
                mouse_delta += mouse_event.delta;
            }
            mouse_delta *= controller.sensitivity;

            input.pitch = (input.pitch - mouse_delta.y)
                .clamp(-FRAC_PI_2 + ANGLE_EPSILON, FRAC_PI_2 - ANGLE_EPSILON);
            input.yaw = input.yaw - mouse_delta.x;
        }

        input.movement = Vec3::new(
            get_axis(&key_input, controller.key_right, controller.key_left),
            get_axis(&key_input, controller.key_up, controller.key_down),
            get_axis(&key_input, controller.key_forward, controller.key_back),
        );
        input.sprint = key_input.pressed(controller.key_sprint);
        input.jump = key_input.pressed(controller.key_jump);
        input.fly = key_input.just_pressed(controller.key_fly);
        input.crouch = key_input.pressed(controller.key_crouch);
    }
}

pub fn fps_controller_look(mut query: Query<(&mut FpsController, &FpsControllerInput)>) {
    for (mut controller, input) in query.iter_mut() {
        controller.pitch = input.pitch;
        controller.yaw = input.yaw;
    }
}

pub fn fps_controller_move(
    time: Res<Time>,
    physics_context: Res<RapierContext>,
    mut query: Query<(
        Entity,
        &FpsControllerInput,
        &mut FpsController,
        &Collider,
        &mut Transform,
        &mut Velocity,
    )>,
) {
    let dt = time.delta_seconds();

    for (entity, input, mut controller, collider, transform, mut velocity) in query.iter_mut() {
        if input.fly {
            controller.move_mode = match controller.move_mode {
                MoveMode::Noclip => MoveMode::Ground,
                MoveMode::Ground => MoveMode::Noclip,
            }
        }

        let orientation = look_quat(input.pitch, input.yaw);
        let right = orientation * Vec3::X;
        let forward = orientation * -Vec3::Z;
        let position = transform.translation;

        match controller.move_mode {
            MoveMode::Noclip => {
                if input.movement == Vec3::ZERO {
                    let friction = controller.fly_friction.clamp(0.0, 1.0);
                    controller.velocity *= 1.0 - friction;
                    if controller.velocity.length_squared() < 1e-6 {
                        controller.velocity = Vec3::ZERO;
                    }
                } else {
                    let fly_speed = if input.sprint {
                        controller.fast_fly_speed
                    } else {
                        controller.fly_speed
                    };
                    controller.velocity = input.movement.normalize() * fly_speed;
                }
                velocity.linvel = controller.velocity.x * right
                    + controller.velocity.y * Vec3::Y
                    + controller.velocity.z * forward;
            }

            MoveMode::Ground => {
                if let Some(capsule) = collider.as_capsule() {
                    let capsule = capsule.raw;
                    let mut start_velocity = controller.velocity;
                    let mut end_velocity = start_velocity;
                    let lateral_speed = start_velocity.xz().length();

                    // Capsule cast downwards to find ground
                    // Better than single raycast as it handles when you are near the edge of a surface
                    let mut ground_hit = None;
                    let cast_capsule = Collider::capsule(
                        capsule.segment.a.into(),
                        capsule.segment.b.into(),
                        capsule.radius * 1.0625,
                    );
                    let cast_velocity = Vec3::Y * -1.0;
                    let max_distance = 0.125;
                    // Avoid self collisions
                    let groups = QueryFilter::default().exclude_rigid_body(entity);

                    if let Some((_handle, hit)) = physics_context.cast_shape(
                        position,
                        orientation,
                        cast_velocity,
                        &cast_capsule,
                        max_distance,
                        groups,
                    ) {
                        ground_hit = Some(hit);
                    }

                    let mut wish_direction = input.movement.z * controller.forward_speed * forward
                        + input.movement.x * controller.side_speed * right;
                    let mut wish_speed = wish_direction.length();
                    if wish_speed > 1e-6 {
                        // Avoid division by zero
                        wish_direction /= wish_speed; // Effectively normalize, avoid length computation twice
                    }

                    let max_speed = if input.sprint {
                        controller.run_speed
                    } else {
                        controller.walk_speed
                    };

                    wish_speed = f32::min(wish_speed, max_speed);

                    if let Some(_ground_hit) = ground_hit {
                        // Only apply friction after at least one tick, allows b-hopping without losing speed
                        if controller.ground_tick >= 1 {
                            if lateral_speed > controller.friction_cutoff {
                                friction(
                                    lateral_speed,
                                    controller.friction,
                                    controller.stop_speed,
                                    dt,
                                    &mut end_velocity,
                                );
                            } else {
                                end_velocity.x = 0.0;
                                end_velocity.z = 0.0;
                            }
                            end_velocity.y = 0.0;
                        }
                        accelerate(
                            wish_direction,
                            wish_speed,
                            controller.accel,
                            dt,
                            &mut end_velocity,
                        );
                        if input.jump {
                            // Simulate one update ahead, since this is an instant velocity change
                            start_velocity.y = controller.jump_speed;
                            end_velocity.y = start_velocity.y - controller.gravity * dt;
                        }
                        // Increment ground tick but cap at max value
                        controller.ground_tick = controller.ground_tick.saturating_add(1);
                    } else {
                        controller.ground_tick = 0;
                        wish_speed = f32::min(wish_speed, controller.air_speed_cap);
                        accelerate(
                            wish_direction,
                            wish_speed,
                            controller.air_acceleration,
                            dt,
                            &mut end_velocity,
                        );
                        end_velocity.y -= controller.gravity * dt;
                        let air_speed = end_velocity.xz().length();
                        if air_speed > controller.max_air_speed {
                            let ratio = controller.max_air_speed / air_speed;
                            end_velocity.x *= ratio;
                            end_velocity.z *= ratio;
                        }
                    }

                    // At this point our collider may be intersecting with the ground
                    // Fix up our collider by offsetting it to be flush with the ground
                    // if end_vel.y < -1e6 {
                    //     if let Some(ground_hit) = ground_hit {
                    //         let normal = Vec3::from(*ground_hit.normal2);
                    //         next_translation += normal * ground_hit.toi;
                    //     }
                    // }

                    controller.velocity = end_velocity;
                    velocity.linvel = (start_velocity + end_velocity) * 0.5;
                }
            }
        }
    }
}

fn look_quat(pitch: f32, yaw: f32) -> Quat {
    Quat::from_euler(EulerRot::ZYX, 0.0, yaw, pitch)
}

fn friction(lateral_speed: f32, friction: f32, stop_speed: f32, dt: f32, velocity: &mut Vec3) {
    let control = f32::max(lateral_speed, stop_speed);
    let drop = control * friction * dt;
    let new_speed = f32::max((lateral_speed - drop) / lateral_speed, 0.0);
    velocity.x *= new_speed;
    velocity.z *= new_speed;
}

fn accelerate(wish_dir: Vec3, wish_speed: f32, accel: f32, dt: f32, velocity: &mut Vec3) {
    let velocity_projection = Vec3::dot(*velocity, wish_dir);
    let add_speed = wish_speed - velocity_projection;
    if add_speed <= 0.0 {
        return;
    }

    let accel_speed = f32::min(accel * wish_speed * dt, add_speed);
    let wish_direction = wish_dir * accel_speed;
    velocity.x += wish_direction.x;
    velocity.z += wish_direction.z;
}

fn get_pressed(key_input: &Res<Input<KeyCode>>, key: KeyCode) -> f32 {
    if key_input.pressed(key) {
        1.0
    } else {
        0.0
    }
}

fn get_axis(key_input: &Res<Input<KeyCode>>, key_pos: KeyCode, key_neg: KeyCode) -> f32 {
    get_pressed(key_input, key_pos) - get_pressed(key_input, key_neg)
}

// ██████╗ ███████╗███╗   ██╗██████╗ ███████╗██████╗
// ██╔══██╗██╔════╝████╗  ██║██╔══██╗██╔════╝██╔══██╗
// ██████╔╝█████╗  ██╔██╗ ██║██║  ██║█████╗  ██████╔╝
// ██╔══██╗██╔══╝  ██║╚██╗██║██║  ██║██╔══╝  ██╔══██╗
// ██║  ██║███████╗██║ ╚████║██████╔╝███████╗██║  ██║
// ╚═╝  ╚═╝╚══════╝╚═╝  ╚═══╝╚═════╝ ╚══════╝╚═╝  ╚═╝

pub fn fps_controller_render(
    logical_query: Query<
        (&Transform, &Collider, &FpsController, &LogicalPlayer),
        With<LogicalPlayer>,
    >,
    mut render_query: Query<(&mut Transform, &RenderPlayer), Without<LogicalPlayer>>,
) {
    // TODO: inefficient O(N^2) loop, use hash map?
    for (logical_transform, collider, controller, logical_player_id) in logical_query.iter() {
        if let Some(capsule) = collider.as_capsule() {
            for (mut render_transform, render_player_id) in render_query.iter_mut() {
                if logical_player_id.0 != render_player_id.0 {
                    continue;
                }
                // TODO: let this be more configurable
                let camera_height = capsule.segment().b().y + capsule.radius() * 0.75;
                render_transform.translation =
                    logical_transform.translation + Vec3::Y * camera_height;
                render_transform.rotation = look_quat(controller.pitch, controller.yaw);
            }
        }
    }
}
