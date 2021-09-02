use macroquad::prelude::*;
use crate::aabb::AABB;

pub const GRAVITY: f32 = -0.56;
pub const EPSILON: f32 = 0.01;

/// Moves an entity one axis at a time and adjusts position and velocity to
/// account for collisions. By moving and checking each axis one at a time,
/// there is no need for a slide vector. Returns the velocity that was actually
/// moved.
fn move_and_collide(
    position: &mut Vec3,
    radius_bounds: Vec3,
    velocity: Vec3,
    aabbs: &Vec<AABB>
) -> Vec3 {
    // This is the ideal velocity that will be equal to starting velocity if
    // there are no collisions on any axis
    let mut moved_velocity = velocity;

    // For each axis, move, collide, move back out, reset velocity
    let axes = [Vec3::X, Vec3::Y, Vec3::Z];
    for (movement_axis, axis) in axes.iter().enumerate() {
        let movement = moved_velocity[movement_axis];
        let sign = movement.signum();
        let desired_axis_velocity = *axis * movement;
        let mut moved = AABB::from_box(
            *position + desired_axis_velocity,
            radius_bounds
        );

        for other_aabb in aabbs.iter() {
            if moved.is_colliding(*other_aabb) {
                let collision_depth = moved.get_collision_depth(*other_aabb);
                let depth_on_this_axis_only = collision_depth[movement_axis];

                moved_velocity[movement_axis] += 
                    -sign * (depth_on_this_axis_only + EPSILON);
                moved = AABB::from_box(
                    *position + desired_axis_velocity,
                    radius_bounds
                );

                // Stop moving in directions that result in a hard stop
                if moved_velocity[movement_axis].abs() <= EPSILON {
                    moved_velocity[movement_axis] = 0.0;
                    // Already stopped, don't continue to look at other AABBs
                    break;
                }
            }
        }
    }

    // Move the entity for real (already collided in every direction)
    *position += moved_velocity;

    moved_velocity
}

/// This is for handling physics such as sliding, swimming, stairs, air, etc.
pub fn physics_move(
    position: &mut Vec3,
    radius_bounds: Vec3,
    velocity: &mut Vec3,
    aabbs: &Vec<AABB>
) {
    // Apply gravity first since a collision will trigger the velocity to zero
    // out (don't fall down if on floor)
    *velocity += Vec3::Y * GRAVITY * get_frame_time();

    // Attempt to move by velocity
    let actual_moved_velocity = move_and_collide(
        position,
        radius_bounds,
        *velocity,
        aabbs
    );

    // Now handle physics now that the entity has been safely moved
    for axis in 0 .. 3 {
        // Update the actual velocity so that movement does not continue on
        // axes that result in continued collision
        if actual_moved_velocity[axis].abs() < velocity[axis].abs() {
            velocity[axis] = 0.0;
        }
    }

    let grounded = velocity.y <= 0.0;

    // Handle physics materials
    if grounded {
        *velocity *= Vec3::ONE * 0.95;
    }
}
