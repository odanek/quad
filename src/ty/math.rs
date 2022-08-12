use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

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
pub type Rad = cgm::Rad<f32>;
pub type Deg = cgm::Deg<f32>;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
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

impl<T: PartialEq> Add<Vec2> for Size<T>
where
    T: Add<f32, Output = T>,
{
    type Output = Size<T>;

    fn add(self, rhs: Vec2) -> Self::Output {
        Self {
            width: self.width + rhs.x,
            height: self.height + rhs.y,
        }
    }
}

impl<T: PartialEq> AddAssign<Vec2> for Size<T>
where
    T: AddAssign<f32>,
{
    fn add_assign(&mut self, rhs: Vec2) {
        self.width += rhs.x;
        self.height += rhs.y;
    }
}

impl<T: PartialEq> Sub<Vec2> for Size<T>
where
    T: Sub<f32, Output = T>,
{
    type Output = Size<T>;

    fn sub(self, rhs: Vec2) -> Self::Output {
        Self {
            width: self.width - rhs.x,
            height: self.height - rhs.y,
        }
    }
}

impl<T: PartialEq> SubAssign<Vec2> for Size<T>
where
    T: SubAssign<f32>,
{
    fn sub_assign(&mut self, rhs: Vec2) {
        self.width -= rhs.x;
        self.height -= rhs.y;
    }
}

impl<T: PartialEq> Mul<f32> for Size<T>
where
    T: Mul<f32, Output = T>,
{
    type Output = Size<T>;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::Output {
            width: self.width * rhs,
            height: self.height * rhs,
        }
    }
}

impl<T: PartialEq> MulAssign<f32> for Size<T>
where
    T: MulAssign<f32>,
{
    fn mul_assign(&mut self, rhs: f32) {
        self.width *= rhs;
        self.height *= rhs;
    }
}

impl<T: PartialEq> Div<f32> for Size<T>
where
    T: Div<f32, Output = T>,
{
    type Output = Size<T>;

    fn div(self, rhs: f32) -> Self::Output {
        Self::Output {
            width: self.width / rhs,
            height: self.height / rhs,
        }
    }
}

impl<T: PartialEq> DivAssign<f32> for Size<T>
where
    T: DivAssign<f32>,
{
    fn div_assign(&mut self, rhs: f32) {
        self.width /= rhs;
        self.height /= rhs;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size_ops() {
        type SizeF = Size<f32>;

        assert_eq!(
            SizeF::new(10., 10.) + Vec2::new(10., 10.),
            SizeF::new(20., 20.)
        );
        assert_eq!(
            SizeF::new(20., 20.) - Vec2::new(10., 10.),
            SizeF::new(10., 10.)
        );
        assert_eq!(SizeF::new(10., 10.) * 2., SizeF::new(20., 20.));
        assert_eq!(SizeF::new(20., 20.) / 2., SizeF::new(10., 10.));

        let mut size = SizeF::new(10., 10.);

        size += Vec2::new(10., 10.);

        assert_eq!(size, SizeF::new(20., 20.));
    }
}
