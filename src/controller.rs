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
    /// If the dot product (alignment) of the normal of the surface and the upward vector,
    /// which is a value from [-1, 1], is greater than this value, ground movement is applied
    pub traction_normal_cutoff: f32,
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
            traction_normal_cutoff: 0.7,
            friction_speed_cutoff: 0.1,
            fly_friction: 0.5,
            pitch: 0.0,
            yaw: 0.0,
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

        match controller.move_mode {
            MoveMode::Noclip => {
                if input.movement == Vec3::ZERO {
                    let friction = controller.fly_friction.clamp(0.0, 1.0);
                    velocity.linvel *= 1.0 - friction;
                    if velocity.linvel.length_squared() < f32::EPSILON {
                        velocity.linvel = Vec3::ZERO;
                    }
                } else {
                    let fly_speed = if input.sprint {
                        controller.fast_fly_speed
                    } else {
                        controller.fly_speed
                    };
                    let mut move_to_world = Mat3::from_euler(EulerRot::YXZ, input.yaw, input.pitch, 0.0);
                    move_to_world.z_axis *= -1.0; // Forward is -Z
                    move_to_world.y_axis = Vec3::Y; // Vertical movement aligned with world up
                    velocity.linvel = move_to_world * input.movement * fly_speed;
                }
            }
            MoveMode::Ground => {
                if let Some(capsule) = collider.as_capsule() {
                    // Capsule cast downwards to find ground
                    // Better than a ray cast as it handles when you are near the edge of a surface
                    let capsule = capsule.raw;
                    let cast_capsule = Collider::capsule(
                        capsule.segment.a.into(), capsule.segment.b.into(),
                        capsule.radius * 0.9,
                    );
                    // Avoid self collisions
                    let cast_groups = QueryFilter::default().exclude_rigid_body(entity);
                    let ground_cast = physics_context.cast_shape(
                        transform.translation, transform.rotation,
                        -Vec3::Y,
                        &cast_capsule,
                        0.125,
                        cast_groups,
                    );

                    let speeds = Vec3::new(controller.side_speed, 0.0, controller.forward_speed);
                    let mut move_to_world = Mat3::from_axis_angle(Vec3::Y, input.yaw);
                    move_to_world.z_axis *= -1.0; // Forward is -Z
                    let mut wish_direction = move_to_world * (input.movement * speeds);
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

                    if let Some((_, hit)) = ground_cast {
                        let has_traction = Vec3::dot(hit.normal1, Vec3::Y) > controller.traction_normal_cutoff;

                        // Only apply friction after at least one tick, allows b-hopping without losing speed
                        if controller.ground_tick >= 1 && has_traction {
                            let lateral_speed = velocity.linvel.xz().length();
                            if lateral_speed > controller.friction_speed_cutoff {
                                let control = f32::max(lateral_speed, controller.stop_speed);
                                let drop = control * controller.friction * dt;
                                let new_speed = f32::max((lateral_speed - drop) / lateral_speed, 0.0);
                                velocity.linvel.x *= new_speed;
                                velocity.linvel.z *= new_speed;
                            } else {
                                velocity.linvel = Vec3::ZERO;
                            }
                            if controller.ground_tick == 1 {
                                velocity.linvel.y = 0.0;
                            }
                        }

                        let mut add = acceleration(
                            wish_direction,
                            wish_speed,
                            controller.acceleration,
                            velocity.linvel,
                            dt,
                        );
                        if !has_traction {
                            add.y -= controller.gravity * dt;
                        }
                        velocity.linvel += add;

                        if has_traction {
                            let linvel = velocity.linvel;
                            velocity.linvel -= Vec3::dot(linvel, hit.normal1) * hit.normal1;

                            if input.jump {
                                velocity.linvel.y = controller.jump_speed;
                            }
                        }

                        // Increment ground tick but cap at max value
                        controller.ground_tick = controller.ground_tick.saturating_add(1);
                    } else {
                        controller.ground_tick = 0;
                        wish_speed = f32::min(wish_speed, controller.air_speed_cap);

                        let mut add = acceleration(
                            wish_direction,
                            wish_speed,
                            controller.air_acceleration,
                            velocity.linvel,
                            dt,
                        );
                        add.y = -controller.gravity * dt;
                        velocity.linvel += add;

                        let air_speed = velocity.linvel.xz().length();
                        if air_speed > controller.max_air_speed {
                            let ratio = controller.max_air_speed / air_speed;
                            velocity.linvel.x *= ratio;
                            velocity.linvel.z *= ratio;
                        }
                    }

                    /* Crouching */

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

                    // Prevent falling off ledges
                    if controller.ground_tick >= 1 && input.crouch {
                        for _ in 0..2 {
                            // Find the component of our velocity that is overhanging and subtract it off
                            let overhang = overhang_component(entity, transform.as_ref(), physics_context.as_ref(), velocity.linvel, dt);
                            if let Some(overhang) = overhang {
                                velocity.linvel -= overhang;
                            }
                        }
                        // If we are still overhanging consider unsolvable and freeze
                        if overhang_component(entity, transform.as_ref(), physics_context.as_ref(), velocity.linvel, dt).is_some() {
                            velocity.linvel = Vec3::ZERO;
                        }
                    }
                }
            }
        }
    }
}

fn overhang_component(entity: Entity, transform: &Transform, physics_context: &RapierContext, velocity: Vec3, dt: f32) -> Option<Vec3> {
    // Cast a segment (zero radius on capsule) from our next position back towards us
    // If there is a ledge in front of us we will hit the edge of it
    // We can use the normal of the hit to subtract off the component that is overhanging
    let cast_capsule = Collider::capsule(Vec3::Y * 0.125, -Vec3::Y * 0.125, 0.0);
    let filter = QueryFilter::default().exclude_rigid_body(entity);
    let future_position = transform.translation + velocity * dt;
    let cast = physics_context.cast_shape(
        future_position, transform.rotation,
        -velocity,
        &cast_capsule,
        0.5,
        filter,
    );
    if let Some((_, toi)) = cast {
        let cast = physics_context.cast_ray(
            future_position + Vec3::Y * 0.125, -Vec3::Y,
            0.375,
            false,
            filter,
        );
        // Make sure that this is actually a ledge, e.g. there is no ground in front of us
        if cast.is_none() {
            let normal = -toi.normal1;
            let alignment = Vec3::dot(velocity, normal);
            return Some(alignment * normal);
        }
    }
    None
}

fn acceleration(wish_direction: Vec3, wish_speed: f32, acceleration: f32, velocity: Vec3, dt: f32) -> Vec3 {
    let velocity_projection = Vec3::dot(velocity, wish_direction);
    let add_speed = wish_speed - velocity_projection;
    if add_speed <= 0.0 {
        return Vec3::ZERO;
    }

    let acceleration_speed = f32::min(acceleration * wish_speed * dt, add_speed);
    wish_direction * acceleration_speed
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
