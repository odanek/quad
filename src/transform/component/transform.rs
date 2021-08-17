use crate::ty::{Quaternion, Vec3};

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quaternion,
    pub scale: Vec3,
}
