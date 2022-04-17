use std::ops::Mul;

use crate::{
    ecs::Component,
    ty::{Mat3, Mat4, Rad, Vec3},
};
use cgm::{ElementWise, Zero};

use super::global_transform::GlobalTransform;

#[derive(Debug, PartialEq, Clone, Copy, Component)]
pub struct Transform {
    pub translation: Vec3,
    pub rotation: Rad,
    pub scale: Vec3,
}

pub const IDENTITY_SCALE: Vec3 = Vec3::new(1.0, 1.0, 1.0);

impl Transform {
    #[inline]
    pub fn from_xy(x: f32, y: f32) -> Self {
        Self::from_translation(Vec3::new(x, y, 0.0))
    }

    #[inline]
    pub fn from_xyz(x: f32, y: f32, z: f32) -> Self {
        Self::from_translation(Vec3::new(x, y, z))
    }

    #[inline]
    pub const fn identity() -> Self {
        Transform {
            translation: Vec3::ZERO,
            rotation: Rad::new(0.0),
            scale: IDENTITY_SCALE,
        }
    }

    #[inline]
    pub fn from_translation(translation: Vec3) -> Self {
        Transform {
            translation,
            ..Default::default()
        }
    }

    #[inline]
    pub fn from_rotation(rotation: Rad) -> Self {
        Transform {
            rotation,
            ..Default::default()
        }
    }

    #[inline]
    pub fn from_scale(scale: Vec3) -> Self {
        Transform {
            scale,
            ..Default::default()
        }
    }

    #[inline]
    pub fn compute_matrix(&self) -> Mat4 {
        // TODO Simplify
        let rotation_matrix = Mat3::from_rotation_z(self.rotation);
        Mat4::from_cols(
            rotation_matrix.x.extend(0.0) * self.scale.x,
            rotation_matrix.y.extend(0.0) * self.scale.y,
            rotation_matrix.z.extend(0.0) * self.scale.z,
            self.translation.extend(1.0),
        )
    }

    #[inline]
    pub fn rotate(&mut self, rotation: Rad) {
        self.rotation += rotation;
    }

    #[inline]
    pub fn back(&self) -> Vec3 {
        Vec3::Z
    }

    #[inline]
    pub fn mul_transform(&self, transform: Transform) -> Self {
        let translation = self * transform.translation;
        let rotation = self.rotation + transform.rotation;
        let scale = self.scale.mul_element_wise(transform.scale);
        Transform {
            translation,
            rotation,
            scale,
        }
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::identity()
    }
}

impl From<GlobalTransform> for Transform {
    fn from(transform: GlobalTransform) -> Self {
        Self {
            translation: transform.translation,
            rotation: transform.rotation,
            scale: transform.scale,
        }
    }
}

impl Mul<Transform> for Transform {
    type Output = Transform;

    #[inline]
    fn mul(self, transform: Transform) -> Self::Output {
        self.mul_transform(transform)
    }
}

impl Mul<Vec3> for Transform {
    type Output = Vec3;

    #[allow(clippy::op_ref)]
    #[inline]
    fn mul(self, value: Vec3) -> Self::Output {
        &self * value
    }
}

impl Mul<Vec3> for &Transform {
    type Output = Vec3;

    #[inline]
    fn mul(self, value: Vec3) -> Self::Output {
        // TODO Simplify
        Vec3::from_homogeneous(self.compute_matrix() * value.to_homogeneous())
    }
}
