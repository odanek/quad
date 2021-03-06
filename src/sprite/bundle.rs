use crate::{
    asset::{Handle, HandleId},
    ecs::Bundle,
    render::{
        texture::{Image, DEFAULT_IMAGE_HANDLE},
        view::Visibility,
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
    /// User indication of whether an entity is visible
    pub visibility: Visibility,
}

impl Default for SpriteBundle {
    fn default() -> Self {
        Self {
            sprite: Default::default(),
            transform: Default::default(),
            global_transform: Default::default(),
            texture: Handle::weak(HandleId::with_id::<Image>(DEFAULT_IMAGE_HANDLE)),
            visibility: Default::default(),
        }
    }
}
/// A Bundle of components for drawing a single sprite from a sprite sheet (also referred
/// to as a `TextureAtlas`)
#[derive(Bundle, Clone, Default)]
pub struct SpriteSheetBundle {
    /// The specific sprite from the texture atlas to be drawn
    pub sprite: TextureAtlasSprite,
    /// A handle to the texture atlas that holds the sprite images
    pub texture_atlas: Handle<TextureAtlas>,
    /// Data pertaining to how the sprite is drawn on the screen
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    /// User indication of whether an entity is visible
    pub visibility: Visibility,
}
