use std::ops::{Div, DivAssign, Mul, MulAssign};

use super::Val;

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct UiRect {
    pub left: Val,
    pub right: Val,
    pub top: Val,
    pub bottom: Val,
}

impl Default for UiRect {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl UiRect {
    pub const DEFAULT: Self = Self {
        left: Val::DEFAULT,
        right: Val::DEFAULT,
        top: Val::DEFAULT,
        bottom: Val::DEFAULT,
    };

    pub fn new(left: Val, right: Val, top: Val, bottom: Val) -> Self {
        UiRect {
            left,
            right,
            top,
            bottom,
        }
    }

    pub fn all(value: Val) -> Self {
        UiRect {
            left: value,
            right: value,
            top: value,
            bottom: value,
        }
    }

    pub fn horizontal(value: Val) -> Self {
        UiRect {
            left: value,
            right: value,
            ..Default::default()
        }
    }

    pub fn vertical(value: Val) -> Self {
        UiRect {
            top: value,
            bottom: value,
            ..Default::default()
        }
    }

    pub fn left(value: Val) -> Self {
        UiRect {
            left: value,
            ..Default::default()
        }
    }

    pub fn right(value: Val) -> Self {
        UiRect {
            right: value,
            ..Default::default()
        }
    }

    pub fn top(value: Val) -> Self {
        UiRect {
            top: value,
            ..Default::default()
        }
    }

    pub fn bottom(value: Val) -> Self {
        UiRect {
            bottom: value,
            ..Default::default()
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub struct Size {
    pub width: Val,
    pub height: Val,
}

impl Size {
    pub const fn new(width: Val, height: Val) -> Self {
        Size { width, height }
    }

    pub const AUTO: Size = Size {
        width: Val::Auto,
        height: Val::Auto,
    };

    pub const UNDEFINED: Size = Size {
        width: Val::Undefined,
        height: Val::Undefined,
    };
}

impl From<(Val, Val)> for Size {
    fn from(vals: (Val, Val)) -> Self {
        Self {
            width: vals.0,
            height: vals.1,
        }
    }
}

impl Mul<f32> for Size {
    type Output = Size;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::Output {
            width: self.width * rhs,
            height: self.height * rhs,
        }
    }
}

impl MulAssign<f32> for Size {
    fn mul_assign(&mut self, rhs: f32) {
        self.width *= rhs;
        self.height *= rhs;
    }
}

impl Div<f32> for Size {
    type Output = Size;

    fn div(self, rhs: f32) -> Self::Output {
        Self::Output {
            width: self.width / rhs,
            height: self.height / rhs,
        }
    }
}

impl DivAssign<f32> for Size {
    fn div_assign(&mut self, rhs: f32) {
        self.width /= rhs;
        self.height /= rhs;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_size_from() {
        let size: Size = (Val::Px(20.), Val::Px(30.)).into();

        assert_eq!(
            size,
            Size {
                width: Val::Px(20.),
                height: Val::Px(30.),
            }
        );
    }

    #[test]
    fn test_size_mul() {
        assert_eq!(
            Size::new(Val::Px(10.), Val::Px(10.)) * 2.,
            Size::new(Val::Px(20.), Val::Px(20.))
        );

        let mut size = Size::new(Val::Px(10.), Val::Px(10.));
        size *= 2.;
        assert_eq!(size, Size::new(Val::Px(20.), Val::Px(20.)));
    }

    #[test]
    fn test_size_div() {
        assert_eq!(
            Size::new(Val::Px(20.), Val::Px(20.)) / 2.,
            Size::new(Val::Px(10.), Val::Px(10.))
        );

        let mut size = Size::new(Val::Px(20.), Val::Px(20.));
        size /= 2.;
        assert_eq!(size, Size::new(Val::Px(10.), Val::Px(10.)));
    }
}
