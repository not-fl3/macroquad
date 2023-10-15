use glam::{vec3, vec4, Mat4, Vec3, Vec4, Vec4Swizzles};

use crate::{camera::Camera, scene::AABB};

#[derive(Debug, Default, Clone, Copy)]
pub struct Plane {
    pub normal: Vec3,
    pub distance: f32,
}
impl Plane {
    pub fn new(normal: Vec3, distance: f32) -> Plane {
        Plane { normal, distance }
    }

    pub fn from_point_and_normal(point: Vec3, normal: Vec3) -> Plane {
        let normal = normal.normalize();
        let distance = normal.dot(point);
        Plane { normal, distance }
    }

    pub fn signed_distance_to_plane(&self, point: Vec3) -> f32 {
        self.normal.dot(point) - self.distance
    }

    pub fn clip(&self, aabb: AABB) -> bool {
        let points = [
            vec3(aabb.min.x, aabb.min.y, aabb.min.z),
            vec3(aabb.min.x, aabb.max.y, aabb.min.z),
            vec3(aabb.max.x, aabb.max.y, aabb.min.z),
            vec3(aabb.max.x, aabb.min.y, aabb.min.z),
            vec3(aabb.min.x, aabb.min.y, aabb.max.z),
            vec3(aabb.min.x, aabb.max.y, aabb.max.z),
            vec3(aabb.max.x, aabb.max.y, aabb.max.z),
            vec3(aabb.max.x, aabb.min.y, aabb.max.z),
        ];
        points.into_iter().any(|p| self.signed_distance_to_plane(*p) > 0.0)
    }
}

pub fn projection_planes(camera: &Camera) -> [Plane; 6] {
    let camera_direction = (camera.target - camera.position).normalize();
    let camera_right = camera_direction.cross(camera.up);
    let far = camera_direction * camera.z_far;
    let near = camera_direction * camera.z_near;

    let (proj, view) = camera.proj_view();
    let inv = (proj * view).inverse();
    let mut ndc = [
        vec4(-1.0, 0.0, 1.0, 1.0),
        vec4(1.0, 0.0, 1.0, 1.0),
        vec4(0.0, 1.0, 1.0, 1.0),
        vec4(0.0, -1.0, 1.0, 1.0),
    ];
    for ndc in &mut ndc {
        *ndc = inv * *ndc;
        *ndc /= ndc.w;
    }

    let planes = [
        Plane::from_point_and_normal(camera.position + near, camera_direction),
        Plane::from_point_and_normal(camera.position + far, -camera_direction),
        Plane::from_point_and_normal(
            camera.position,
            (ndc[0].xyz() - camera.position).cross(camera.up),
        ),
        Plane::from_point_and_normal(
            camera.position,
            camera.up.cross(ndc[1].xyz() - camera.position),
        ),
        Plane::from_point_and_normal(
            camera.position,
            (ndc[2].xyz() - camera.position).cross(camera_right),
        ),
        Plane::from_point_and_normal(
            camera.position,
            camera_right.cross(ndc[3].xyz() - camera.position),
        ),
    ];

    planes
}
