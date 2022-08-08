use bevy::{
    math::Vec3Swizzles,
    prelude::*,
};
use bevy_rapier3d::prelude::*;

pub struct FpsControllerPlugin;

impl Plugin for FpsControllerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system(player_look)
            .add_system(player_move);
    }
}

pub enum MoveMode {
    Noclip,
    Ground,
}

#[derive(Component)]
pub struct FpsControllerInput {
    fly: bool,
    sprint: bool,
    jump: bool,
    pitch: f32,
    yaw: f32,
    movement: Vec3,
}

#[derive(Component)]
pub struct FpsController {
    pub move_mode: MoveMode,
    pub gravity: f32,
    pub walk_speed: f32,
    pub run_speed: f32,
    pub fwd_speed: f32,
    pub side_speed: f32,
    pub air_speed_cap: f32,
    pub air_accel: f32,
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
}

impl Default for FpsController {
    fn default() -> Self {
        Self {
            move_mode: MoveMode::Noclip,
            fly_speed: 10.0,
            fast_fly_speed: 30.0,
            gravity: 23.0,
            walk_speed: 10.0,
            run_speed: 30.0,
            fwd_speed: 30.0,
            side_speed: 30.0,
            air_speed_cap: 2.0,
            air_accel: 20.0,
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
        }
    }
}

pub fn player_look(
    mut query: Query<(&mut FpsController, &FpsControllerInput)>
) {
    for (mut controller, input) in query.iter_mut() {
        controller.pitch = input.pitch;
        controller.yaw = input.yaw;
    }
}

pub fn player_move(
    time: Res<Time>,
    physics_context: Res<RapierContext>,
    mut query: Query<(
        Entity, &FpsControllerInput, &mut FpsController,
        &Collider, &mut Transform, &mut Velocity
    )>,
) {
    let dt = time.delta_seconds();

    for (_entity, input, mut controller, collider, transform, mut velocity) in query.iter_mut() {
        if input.fly {
            controller.move_mode = match controller.move_mode {
                MoveMode::Noclip => MoveMode::Ground,
                MoveMode::Ground => MoveMode::Noclip
            }
        }

        let rot = look_quat(input.pitch, input.yaw);
        let right = rot * Vec3::X;
        let fwd = rot * -Vec3::Z;
        let pos = transform.translation;

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
                    + controller.velocity.z * fwd;
            }

            MoveMode::Ground => {
                if let Some(capsule) = collider.as_capsule() {
                    let capsule = capsule.raw;
                    let mut init_vel = controller.velocity;
                    let mut end_vel = init_vel;
                    let lateral_speed = init_vel.xz().length();

                    // Capsule cast downwards to find ground
                    let mut ground_hit = None;
                    let cast_capsule = Collider::capsule(capsule.segment.a.into(), capsule.segment.b.into(), capsule.radius * 1.0625);
                    let cast_vel = Vec3::Y * -1.0;
                    let max_dist = 0.125;
                    let groups = QueryFilter::default();

                    if let Some((_handle, hit)) = physics_context.cast_shape(
                        pos, rot, cast_vel, &cast_capsule, max_dist, groups,
                        // Filter to prevent self-collisions and collisions with non-solid objects
                        // Some(&|hit_ent| {
                        //     hit_ent != entity && match sensor_query.get(hit_ent) {
                        //         Ok(sensor) => !sensor.0,
                        //         Err(_) => true
                        //     }
                        // }),
                    ) {
                        ground_hit = Some(hit);
                    }

                    let mut wish_dir = input.movement.z * controller.fwd_speed * fwd + input.movement.x * controller.side_speed * right;
                    let mut wish_speed = wish_dir.length();
                    if wish_speed > 1e-6 { // Avoid division by zero
                        wish_dir /= wish_speed; // Effectively normalize, avoid length computation twice
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
                                friction(lateral_speed, controller.friction, controller.stop_speed, dt, &mut end_vel);
                            } else {
                                end_vel.x = 0.0;
                                end_vel.z = 0.0;
                            }
                            end_vel.y = 0.0;
                        }
                        accelerate(wish_dir, wish_speed, controller.accel, dt, &mut end_vel);
                        if input.jump {
                            // Simulate one update ahead, since this is an instant velocity change
                            init_vel.y = controller.jump_speed;
                            end_vel.y = init_vel.y - controller.gravity * dt;
                        }
                        // Increment ground tick but cap at max value
                        controller.ground_tick = controller.ground_tick.saturating_add(1);
                    } else {
                        controller.ground_tick = 0;
                        wish_speed = f32::min(wish_speed, controller.air_speed_cap);
                        accelerate(wish_dir, wish_speed, controller.air_accel, dt, &mut end_vel);
                        end_vel.y -= controller.gravity * dt;
                        let air_speed = end_vel.xz().length();
                        if air_speed > controller.max_air_speed {
                            let ratio = controller.max_air_speed / air_speed;
                            end_vel.x *= ratio;
                            end_vel.z *= ratio;
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

                    controller.velocity = end_vel;
                    velocity.linvel = (init_vel + end_vel) * 0.5;
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
    let vel_proj = Vec3::dot(*velocity, wish_dir);
    let add_speed = wish_speed - vel_proj;
    if add_speed <= 0.0 { return; }

    let accel_speed = f32::min(accel * wish_speed * dt, add_speed);
    let wish_dir = wish_dir * accel_speed;
    velocity.x += wish_dir.x;
    velocity.z += wish_dir.z;
}