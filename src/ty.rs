mod float;
mod future;
mod math;

pub use float::*;
pub use future::*;
pub use math::*;

pub mod prelude {
    pub use crate::ty::{Mat2, Mat3, Mat4, Size, Vec1, Vec2, Vec3, Vec4};
}
