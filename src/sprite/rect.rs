use crate::ty::Vec2;

/// A rectangle defined by two points. There is no defined origin, so 0,0 could be anywhere
/// (top-left, bottom-left, etc)
#[repr(C)]
#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    /// The beginning point of the rect
    pub min: Vec2,
    /// The ending point of the rect
    pub max: Vec2,
}

impl Rect {
    #[inline]
    pub fn new(x0: f32, y0: f32, x1: f32, y1: f32) -> Self {
        Self::from_corners(Vec2::new(x0, y0), Vec2::new(x1, y1))
    }

    #[inline]
    pub fn from_corners(p0: Vec2, p1: Vec2) -> Self {
        Rect {
            min: p0.min_element_wise(p1),
            max: p0.max_element_wise(p1),
        }
    }

    #[inline]
    pub fn width(&self) -> f32 {
        self.max.x - self.min.x
    }

    #[inline]
    pub fn height(&self) -> f32 {
        self.max.y - self.min.y
    }

    #[inline]
    pub fn center(&self) -> Vec2 {
        (self.min + self.max) * 0.5
    }

    #[inline]
    pub fn size(&self) -> Vec2 {
        Vec2::new(self.width(), self.height())
    }

    #[inline]
    pub fn half_size(&self) -> Vec2 {
        self.size() * 0.5
    }

    #[inline]
    pub fn from_center_size(origin: Vec2, size: Vec2) -> Self {
        let half_size = size / 2.;
        Self::from_center_half_size(origin, half_size)
    }

    #[inline]
    pub fn from_center_half_size(origin: Vec2, half_size: Vec2) -> Self {
        Self {
            min: origin - half_size,
            max: origin + half_size,
        }
    }

    #[inline]
    pub fn union(&self, other: Rect) -> Rect {
        Rect {
            min: self.min.min_element_wise(other.min),
            max: self.max.max_element_wise(other.max),
        }
    }

    /// ```
    #[inline]
    pub fn intersect(&self, other: Rect) -> Rect {
        let mut r = Rect {
            min: self.min.max_element_wise(other.min),
            max: self.max.min_element_wise(other.max),
        };
        // Collapse min over max to enforce invariants and ensure e.g. width() or
        // height() never return a negative value.
        r.min = r.min.min_element_wise(r.max);
        r
    }
}
