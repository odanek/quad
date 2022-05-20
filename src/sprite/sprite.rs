use crate::{ecs::Component, render::color::Color, ty::Vec2};

use super::Rect;

#[derive(Component, Debug, Default, Clone)]
#[repr(C)]
pub struct Sprite {
    /// The sprite's color tint
    pub color: Color,
    /// Flip the sprite along the X axis
    pub flip_x: bool,
    /// Flip the sprite along the Y axis
    pub flip_y: bool,
    /// An optional custom size for the sprite that will be used when rendering, instead of the size
    /// of the sprite's image
    pub custom_size: Option<Vec2>,    
    /// Optional rect
    pub rect: Option<Rect>,
}
