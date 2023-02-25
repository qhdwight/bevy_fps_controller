use std::f32::consts::*;

use bevy::{
    input::mouse::MouseMotion,
    math::Vec3Swizzles,
    prelude::*,
};
use bevy_rapier3d::prelude::*;

pub struct FpsControllerPlugin;

#[derive(SystemLabel)]
enum FpsSystemLabel {
    Input,
    Look,
    Move,
    Render,
}

impl Plugin for FpsControllerPlugin {
    fn build(&self, app: &mut App) {
        // TODO: use system piping instead?
        app.add_system_set(SystemSet::new()
            .with_system(fps_controller_input.label(FpsSystemLabel::Input))
            .with_system(fps_controller_look.label(FpsSystemLabel::Look).after(FpsSystemLabel::Input))
            .with_system(fps_controller_move.label(FpsSystemLabel::Move).after(FpsSystemLabel::Look))
            .with_system(fps_controller_render.label(FpsSystemLabel::Render).after(FpsSystemLabel::Move))
        );
    }
}

#[derive(PartialEq)]
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
    pub radius: f32,
    pub gravity: f32,
    pub walk_speed: f32,
    pub run_speed: f32,
    pub forward_speed: f32,
    pub side_speed: f32,
    pub air_speed_cap: f32,
    pub air_acceleration: f32,
    pub max_air_speed: f32,
    pub acceleration: f32,
    pub friction: f32,
    /// If the dot product of the normal of the surface and the upward vector,
    /// which is a value from [-1, 1], is greater than this value, friction will be applied
    pub friction_normal_cutoff: f32,
    pub friction_speed_cutoff: f32,
    pub jump_speed: f32,
    pub fly_speed: f32,
    pub crouched_speed: f32,
    pub crouch_speed: f32,
    pub uncrouch_speed: f32,
    pub height: f32,
    pub upright_height: f32,
    pub crouch_height: f32,
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
            radius: 0.5,
            fly_speed: 10.0,
            fast_fly_speed: 30.0,
            gravity: 23.0,
            walk_speed: 9.0,
            run_speed: 14.0,
            forward_speed: 30.0,
            side_speed: 30.0,
            air_speed_cap: 2.0,
            air_acceleration: 20.0,
            max_air_speed: 15.0,
            crouched_speed: 5.0,
            crouch_speed: 6.0,
            uncrouch_speed: 8.0,
            height: 1.5,
            upright_height: 2.0,
            crouch_height: 1.25,
            acceleration: 10.0,
            friction: 10.0,
            friction_normal_cutoff: 0.7,
            friction_speed_cutoff: 0.1,
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

            input.pitch = (input.pitch - mouse_delta.y).clamp(-FRAC_PI_2 + ANGLE_EPSILON, FRAC_PI_2 - ANGLE_EPSILON);
            input.yaw -= mouse_delta.x;
            if input.yaw.abs() > PI {
                input.yaw = input.yaw.rem_euclid(TAU);
            }
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
        &mut Collider,
        &mut Transform,
        &mut Velocity,
    )>,
) {
    let dt = time.delta_seconds();

    for (entity, input, mut controller, mut collider, transform, mut velocity) in query.iter_mut() {
        if input.fly {
            controller.move_mode = match controller.move_mode {
                MoveMode::Noclip => MoveMode::Ground,
                MoveMode::Ground => MoveMode::Noclip,
            }
        }

        // Change of basis matrix from local move space to world space
        let mut move_to_world = calc_local_to_world(input.yaw, input.pitch);
        move_to_world.y_axis = Vec3::Y; // Vertical movement aligned with world up

        match controller.move_mode {
            MoveMode::Noclip => {
                if input.movement == Vec3::ZERO {
                    let friction = controller.fly_friction.clamp(0.0, 1.0);
                    controller.velocity *= 1.0 - friction;
                    if controller.velocity.length_squared() < f32::EPSILON {
                        controller.velocity = Vec3::ZERO;
                    }
                } else {
                    let fly_speed = if input.sprint {
                        controller.fast_fly_speed
                    } else {
                        controller.fly_speed
                    };
                    controller.velocity = move_to_world * input.movement * fly_speed;
                }
                velocity.linvel = controller.velocity;
            }
            MoveMode::Ground => {
                if let Some(capsule) = collider.as_capsule() {
                    // Capsule cast downwards to find ground
                    // Better than a ray cast as it handles when you are near the edge of a surface
                    let capsule = capsule.raw;
                    let cast_capsule = Collider::capsule(
                        capsule.segment.a.into(),
                        capsule.segment.b.into(),
                        capsule.radius * 0.9375,
                    );
                    // Avoid self collisions
                    let cast_groups = QueryFilter::default().exclude_rigid_body(entity);
                    let ground_hit = physics_context.cast_shape(
                        transform.translation, transform.rotation,
                        -Vec3::Y,
                        &cast_capsule,
                        0.125,
                        cast_groups,
                    );

                    let speeds = Vec3::new(controller.side_speed, 0.0, controller.forward_speed);
                    if let Some((_, hit)) = ground_hit {
                        move_to_world.x_axis -= Vec3::dot(move_to_world.x_axis, hit.normal1) * hit.normal1;
                        move_to_world.z_axis -= Vec3::dot(move_to_world.z_axis, hit.normal1) * hit.normal1;
                    }
                    let mut wish_direction = move_to_world * input.movement * speeds;

                    let mut wish_speed = wish_direction.length();
                    if wish_speed > f32::EPSILON {
                        // Avoid division by zero
                        wish_direction /= wish_speed; // Effectively normalize, avoid length computation twice
                    }

                    let max_speed = if input.crouch {
                        controller.crouched_speed
                    } else if input.sprint {
                        controller.run_speed
                    } else {
                        controller.walk_speed
                    };

                    wish_speed = f32::min(wish_speed, max_speed);

                    if let Some((_, hit)) = ground_hit {
                        let is_flat_ground = {
                            let dot = Vec3::dot(hit.normal1, Vec3::Y);
                            dot > controller.friction_normal_cutoff
                        };
                        // Only apply friction after at least one tick, allows b-hopping without losing speed
                        if controller.ground_tick >= 1 && is_flat_ground {
                            let lateral_speed = controller.velocity.xz().length();
                            if lateral_speed > controller.friction_speed_cutoff {
                                friction(
                                    lateral_speed,
                                    controller.friction,
                                    controller.stop_speed,
                                    dt,
                                    &mut controller.velocity,
                                );
                            } else {
                                controller.velocity.x = 0.0;
                                controller.velocity.z = 0.0;
                            }
                            // controller.velocity.y = 0.0;
                        }
                        accelerate(
                            wish_direction,
                            wish_speed,
                            controller.acceleration,
                            dt,
                            &mut controller.velocity,
                        );
                        if input.jump && is_flat_ground {
                            controller.velocity.y = controller.jump_speed;
                        } else if controller.ground_tick == 0 {
                            controller.velocity.y = 0.0;
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
                            &mut controller.velocity,
                        );
                        controller.velocity.y -= controller.gravity * dt;
                        let air_speed = controller.velocity.xz().length();
                        if air_speed > controller.max_air_speed {
                            let ratio = controller.max_air_speed / air_speed;
                            controller.velocity.x *= ratio;
                            controller.velocity.z *= ratio;
                        }
                    }

                    let crouch_height = controller.crouch_height;
                    let upright_height = controller.upright_height;

                    let crouch_speed = if input.crouch {
                        -controller.crouch_speed
                    } else {
                        controller.uncrouch_speed
                    };
                    controller.height += dt * crouch_speed;
                    controller.height = controller.height.clamp(crouch_height, upright_height);

                    if let Some(mut capsule) = collider.as_capsule_mut() {
                        capsule.set_segment(
                            Vec3::Y * 0.5,
                            Vec3::Y * controller.height,
                        );
                    }

                    // Prevent falling off of ledges
                    // TODO: instead of setting to zero subtract out the part that would make us fall
                    let future_position = transform.translation + Vec3::new(controller.velocity.x, 0.0, controller.velocity.z) * dt;
                    if input.crouch && ground_hit.is_some() && physics_context.cast_shape(
                        future_position,
                        transform.rotation,
                        -Vec3::Y,
                        &cast_capsule,
                        0.125,
                        cast_groups,
                    ).is_none() {
                        controller.velocity.x = 0.0;
                        controller.velocity.z = 0.0;
                    }

                    velocity.linvel = controller.velocity;
                }
            }
        }
    }
}

fn calc_local_to_world(yaw: f32, pitch: f32) -> Mat3 {
    let mut local_to_world = Mat3::from_euler(EulerRot::YXZ, yaw, pitch, 0.0);
    local_to_world.z_axis *= -1.0; // Forward is -Z
    local_to_world
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
    *velocity += wish_direction;
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
                render_transform.translation = logical_transform.translation + Vec3::Y * camera_height;
                render_transform.rotation = Quat::from_euler(EulerRot::YXZ, controller.yaw, controller.pitch, 0.0);
            }
        }
    }
}
