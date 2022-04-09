use cgm::{InnerSpace, Matrix};

use crate::{
    ecs::Component,
    ty::{Mat4, Vec3, Vec4},
};

#[derive(Component, Clone, Debug, Default)]
pub struct Aabb {
    pub center: Vec3,
    pub half_extents: Vec3,
}

impl Aabb {
    pub fn from_min_max(minimum: Vec3, maximum: Vec3) -> Self {
        let center = 0.5 * (maximum + minimum);
        let half_extents = 0.5 * (maximum - minimum);
        Self {
            center,
            half_extents,
        }
    }

    /// Calculate the relative radius of the AABB with respect to a plane
    pub fn relative_radius(&self, p_normal: &Vec3, axes: &[Vec3]) -> f32 {
        let half_extents = self.half_extents;
        Vec3::new(
            p_normal.dot(axes[0]),
            p_normal.dot(axes[1]),
            p_normal.dot(axes[2]),
        )
        .abs()
        .dot(half_extents)
    }

    pub fn min(&self) -> Vec3 {
        self.center - self.half_extents
    }

    pub fn max(&self) -> Vec3 {
        self.center + self.half_extents
    }
}

/// A plane defined by a normal and distance value along the normal
/// Any point p is in the plane if n.p = d
/// For planes defining half-spaces such as for frusta, if n.p > d then p is on the positive side of the plane.
#[derive(Clone, Copy, Debug, Default)]
pub struct Plane {
    pub normal_d: Vec4,
}

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct Frustum {
    pub planes: [Plane; 6],
}

impl Frustum {
    // NOTE: This approach of extracting the frustum planes from the view
    // projection matrix is from Foundations of Game Engine Development 2
    // Rendering by Lengyel. Slight modification has been made for when
    // the far plane is infinite but we still want to cull to a far plane.
    pub fn from_view_projection(
        view_projection: &Mat4,
        view_translation: &Vec3,
        view_backward: &Vec3,
        far: f32,
    ) -> Self {
        let row3 = view_projection.row(3);
        let mut planes = [Plane::default(); 6];
        for (i, plane) in planes.iter_mut().enumerate().take(5) {
            let row = view_projection.row(i / 2);
            plane.normal_d = if (i & 1) == 0 && i != 4 {
                row3 + row
            } else {
                row3 - row
            }
            .normalize();
        }
        let far_center = *view_translation - far * *view_backward;
        planes[5].normal_d = view_backward
            .extend(-view_backward.dot(far_center))
            .normalize();
        Self { planes }
    }

    pub fn intersects_obb(&self, aabb: &Aabb, model_to_world: &Mat4) -> bool {
        let aabb_center_world = *model_to_world * aabb.center.extend(1.0);
        let axes = [
            model_to_world.x.truncate(),
            model_to_world.y.truncate(),
            model_to_world.z.truncate(),
        ];

        for plane in &self.planes {
            let p_normal = plane.normal_d.truncate();
            // TODO Is the relative radius needed for 2D? Just cull based on the camera bounding rect.
            let relative_radius = aabb.relative_radius(&p_normal, &axes);
            if plane.normal_d.dot(aabb_center_world) + relative_radius <= 0.0 {
                return false;
            }
        }
        true
    }
}
