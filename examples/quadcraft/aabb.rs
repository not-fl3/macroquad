use macroquad::prelude::*;

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}

impl AABB {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    #[inline]
    pub fn get_center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    pub fn get_bounds(&self) -> Vec3 {
        vec3(
            self.max.x.max(self.min.x) - self.min.x.min(self.max.x),
            self.max.y.max(self.min.y) - self.min.y.min(self.max.y),
            self.max.z.max(self.min.z) - self.min.z.min(self.max.z),
        )
    }

    pub fn is_colliding(&self, other: AABB) -> bool {
        let depth = self.get_collision_depth(other);
        depth[0] > 0.0 && depth[1] > 0.0 && depth[2] > 0.0
    }

    /// Gets the distance on each axis that is inside the other AABB once a
    /// collision has already been confirmed. Negative distance means there is
    /// no collision.
    #[inline]
    pub fn get_collision_depth(&self, other: AABB) -> Vec3 {
        let mut result = Vec3::ZERO;
        let a = self.get_center();
        let b = other.get_center();

        for i in 0..3 {
            if a[i] < b[i] {
                result[i] = self.max[i] - other.min[i];
            } else {
                result[i] = other.max[i] - self.min[i];
            }
        }

        result
    }

    pub fn from_box(position: Vec3, radius_bounds: Vec3) -> Self {
        Self {
            min: position - radius_bounds,
            max: position + radius_bounds,
        }
    }

    pub fn intersects_ray(&self, origin: Vec3, direction: Vec3) -> f32 {
        let t1 = (self.min.x - origin.x) / direction.x;
        let t2 = (self.max.x - origin.x) / direction.x;
        let t3 = (self.min.y - origin.y) / direction.y;
        let t4 = (self.max.y - origin.y) / direction.y;
        let t5 = (self.min.z - origin.z) / direction.z;
        let t6 = (self.max.z - origin.z) / direction.z;
        let t7 = t1.min(t2).max(t3.min(t4)).max(t5.min(t6));
        let t8 = t1.max(t2).min(t3.max(t4)).min(t5.max(t6));

        if t8 < 0.0 || t7 > t8 {
            -1.0
        } else {
            t7
        }
    }
}

mod test {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_aabb() {
        let aabb = AABB {
            min: Vec3::ZERO,
            max: Vec3::ONE,
        };
        let gold = aabb.clone();
        assert!(gold == aabb);

        let aabb = AABB {
            min: Vec3::ONE,
            max: Vec3::ONE,
        };
        assert!(gold != aabb);
    }

    #[test]
    fn test_get_center() {
        assert!(AABB::new(vec3(-1.0, -1.0, -1.0), Vec3::ONE).get_center() == Vec3::ZERO);
        assert!(AABB::new(vec3(-4.0, -1.0, -1.0), Vec3::ONE).get_center() == vec3(-1.5, 0.0, 0.0));
    }

    #[test]
    fn test_get_collision_depth() {
        let a = AABB::new(vec3(-1.0, -1.0, -1.0), Vec3::ONE);
        let b = AABB::new(vec3(0.5, 0.5, 0.5), vec3(10000.5, 10000.5, 10000.5));

        let depth = a.get_collision_depth(b);

        assert_eq!(depth, vec3(0.5, 0.5, 0.5));

        let c = AABB::new(vec3(500.5, 500.5, 500.5), vec3(10000.5, 10000.5, 10000.5));
        let depth = a.get_collision_depth(c);

        assert_eq!(depth, vec3(-499.5, -499.5, -499.5));
    }

    #[test]
    fn test_is_there_a_collision() {
        let a = AABB::new(vec3(-1.0, -1.0, -1.0), Vec3::ONE);
        let b = a.clone();

        assert_eq!(a.get_collision_depth(b), vec3(2.0, 2.0, 2.0));
        assert_eq!(a.is_colliding(b), true);

        let a = AABB::new(vec3(-1.0, -1.0, -1.0), Vec3::ONE);
        let b = AABB::new(Vec3::ZERO, vec3(1.0, 1.0, 1.0));

        assert_eq!(a.get_collision_depth(b), vec3(1.0, 1.0, 1.0));
        assert_eq!(a.is_colliding(b), true);

        let a = AABB::new(vec3(-1.0, -1.0, -1.0), Vec3::ONE);
        let b = AABB::new(Vec3::ONE * 5.0, vec3(10.0, 10.0, 10.0));

        assert_eq!(a.get_collision_depth(b), vec3(-4.0, -4.0, -4.0));
        assert_eq!(a.is_colliding(b), false);
    }

    #[test]
    fn test_aabb_intersects_ray() {
        let a = AABB::new(vec3(-1.0, -1.0, -1.0), Vec3::ONE);

        assert_eq!(
            a.intersects_ray(vec3(10.0, 10.0, 10.0), vec3(0.0, 1.0, 0.0)),
            -1.0
        );

        assert_eq!(
            a.intersects_ray(vec3(10.0, 0.0, 0.0), vec3(-1.0, 0.0, 0.0)),
            9.0
        );
    }
}
