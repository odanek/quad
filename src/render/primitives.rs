use cgm::{InnerSpace, Matrix};

use crate::{
    ecs::Component,
    ty::{Mat4, Vec3, Vec4},
};

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
}
