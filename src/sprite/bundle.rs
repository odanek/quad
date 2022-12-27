use crate::{
    asset::{Handle, HandleId},
    ecs::Bundle,
    render::{
        texture::{Image, DEFAULT_IMAGE_HANDLE},
        view::{ComputedVisibility, Visibility},
    },
    transform::{GlobalTransform, Transform},
};

use super::{Sprite, TextureAtlas, TextureAtlasSprite};

#[derive(Bundle, Clone)]
pub struct SpriteBundle {
    pub sprite: Sprite,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub texture: Handle<Image>,
    pub visibility: Visibility,
    pub computed_visibility: ComputedVisibility,
}

impl Default for SpriteBundle {
    fn default() -> Self {
        Self {
            sprite: Default::default(),
            transform: Default::default(),
            global_transform: Default::default(),
            texture: Handle::weak(HandleId::with_id::<Image>(DEFAULT_IMAGE_HANDLE)),
            visibility: Default::default(),
            computed_visibility: Default::default(),
        }
    }
}
/// A Bundle of components for drawing a single sprite from a sprite sheet (also referred
/// to as a `TextureAtlas`)
#[derive(Bundle, Clone, Default)]
pub struct SpriteSheetBundle {
    pub sprite: TextureAtlasSprite,
    pub texture_atlas: Handle<TextureAtlas>,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub computed_visibility: ComputedVisibility,
}
