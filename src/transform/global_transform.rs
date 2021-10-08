use std::ops::Mul;

use cgm::{ElementWise, InnerSpace, One, Zero};

use crate::{
    ecs::Component,
    transform::Transform,
    ty::{Mat3, Mat4, Quat, Vec3},
};

use super::transform::IDENTITY_SCALE;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct GlobalTransform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}
impl Component for GlobalTransform {}

impl GlobalTransform {
    #[doc(hidden)]
    #[inline]
    pub fn from_xyz(x: f32, y: f32, z: f32) -> Self {
        Self::from_translation(Vec3::new(x, y, z))
    }

    #[inline]
    pub const fn identity() -> Self {
        GlobalTransform {
            translation: Vec3::ZERO,
            rotation: Quat::ONE,
            scale: IDENTITY_SCALE,
        }
    }

    #[inline]
    pub fn from_matrix(matrix: Mat4) -> Self {
        let (scale, rotation, translation) = matrix.to_scale_quaternion_translation();

        GlobalTransform {
            translation,
            rotation,
            scale,
        }
    }

    #[doc(hidden)]
    #[inline]
    pub fn from_translation(translation: Vec3) -> Self {
        GlobalTransform {
            translation,
            ..Default::default()
        }
    }

    #[doc(hidden)]
    #[inline]
    pub fn from_rotation(rotation: Quat) -> Self {
        GlobalTransform {
            rotation,
            ..Default::default()
        }
    }

    #[inline]
    pub fn from_scale(scale: Vec3) -> Self {
        GlobalTransform {
            scale,
            ..Default::default()
        }
    }

    #[inline]
    pub fn looking_at(mut self, target: Vec3, up: Vec3) -> Self {
        self.look_at(target, up);
        self
    }

    #[inline]
    pub fn compute_matrix(&self) -> Mat4 {
        Mat4::from_scale_quaternion_translation(self.scale, self.rotation, self.translation)
    }

    #[inline]
    pub fn local_x(&self) -> Vec3 {
        self.rotation * Vec3::X
    }

    #[inline]
    pub fn left(&self) -> Vec3 {
        -self.local_x()
    }

    #[inline]
    pub fn right(&self) -> Vec3 {
        self.local_x()
    }

    #[inline]
    pub fn local_y(&self) -> Vec3 {
        self.rotation * Vec3::Y
    }

    #[inline]
    pub fn up(&self) -> Vec3 {
        self.local_y()
    }

    #[inline]
    pub fn down(&self) -> Vec3 {
        -self.local_y()
    }

    #[inline]
    pub fn local_z(&self) -> Vec3 {
        self.rotation * Vec3::Z
    }

    #[inline]
    pub fn forward(&self) -> Vec3 {
        -self.local_z()
    }

    #[inline]
    pub fn back(&self) -> Vec3 {
        self.local_z()
    }

    #[inline]
    pub fn rotate(&mut self, rotation: Quat) {
        self.rotation = rotation * self.rotation;
    }

    #[inline]
    pub fn mul_transform(&self, transform: Transform) -> GlobalTransform {
        let translation = self * transform.translation;
        let rotation = self.rotation * transform.rotation;
        let scale = self.scale.mul_element_wise(transform.scale);
        GlobalTransform {
            translation,
            rotation,
            scale,
        }
    }

    #[inline]
    pub fn apply_non_uniform_scale(&mut self, scale: Vec3) {
        self.scale.mul_assign_element_wise(scale);
    }

    #[inline]
    pub fn look_at(&mut self, target: Vec3, up: Vec3) {
        let forward = (self.translation - target).normalize();
        let right = up.cross(forward).normalize();
        let up = forward.cross(right);
        self.rotation = Mat3::from_cols(right, up, forward).into();
    }
}

impl Default for GlobalTransform {
    fn default() -> Self {
        Self::identity()
    }
}

impl From<Transform> for GlobalTransform {
    fn from(transform: Transform) -> Self {
        Self {
            translation: transform.translation,
            rotation: transform.rotation,
            scale: transform.scale,
        }
    }
}

impl Mul<GlobalTransform> for GlobalTransform {
    type Output = GlobalTransform;

    #[inline]
    fn mul(self, global_transform: GlobalTransform) -> Self::Output {
        self.mul_transform(global_transform.into())
    }
}

impl Mul<Transform> for GlobalTransform {
    type Output = GlobalTransform;

    #[inline]
    fn mul(self, transform: Transform) -> Self::Output {
        self.mul_transform(transform)
    }
}

impl Mul<Vec3> for GlobalTransform {
    type Output = Vec3;

    #[inline]
    #[allow(clippy::op_ref)]
    fn mul(self, value: Vec3) -> Self::Output {
        &self * value
    }
}

impl Mul<Vec3> for &GlobalTransform {
    type Output = Vec3;

    #[inline]
    fn mul(self, value: Vec3) -> Self::Output {
        (self.rotation * value).mul_element_wise(self.scale) + self.translation
    }
}
