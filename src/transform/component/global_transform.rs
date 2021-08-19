use crate::ty::{Quat, Vec3};


#[derive(Debug, PartialEq, Clone, Copy)]
pub struct GlobalTransform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}
