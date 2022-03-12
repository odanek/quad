pub type Vec1 = cgm::Vec1<f32>;
pub type Vec2 = cgm::Vec2<f32>;
pub type Vec2i = cgm::Vec2<i32>;
pub type Vec2u = cgm::Vec2<u32>;
pub type Vec3 = cgm::Vec3<f32>;
pub type Vec4 = cgm::Vec4<f32>;
pub type Mat2 = cgm::Mat2<f32>;
pub type Mat3 = cgm::Mat3<f32>;
pub type Mat4 = cgm::Mat4<f32>;
pub type Quat = cgm::Quat<f32>;

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Size<T: PartialEq = f32> {
    pub width: T,
    pub height: T,
}

impl<T: PartialEq> Size<T> {
    pub fn new(width: T, height: T) -> Self {
        Size { width, height }
    }
}

impl<T: Default + PartialEq> Default for Size<T> {
    fn default() -> Self {
        Self {
            width: Default::default(),
            height: Default::default(),
        }
    }
}
