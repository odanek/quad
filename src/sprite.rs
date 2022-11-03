mod bundle;
mod dynamic_texture_atlas_builder;
mod rect;
mod render;
#[allow(clippy::module_inception)]
mod sprite;
mod texture_atlas;
mod texture_atlas_builder;

pub mod collide_aabb;

pub use bundle::*;
pub use dynamic_texture_atlas_builder::*;
pub use rect::*;
pub use render::*;
pub use sprite::*;
pub use texture_atlas::*;
pub use texture_atlas_builder::*;

use crate::{
    app::{App, RenderStage},
    asset::{Assets, HandleId},
    ecs::IntoSystem,
    pipeline::Transparent2d,
    render::render_resource::{Shader, SpecializedPipelines},
};

pub mod prelude {
    pub use crate::sprite::{
        Rect, Sprite, SpriteBundle, SpriteSheetBundle, TextureAtlas, TextureAtlasSprite,
    };
}

#[derive(Default)]
pub struct SpritePlugin;

// TODO Store these as a resource? Use some non random id?
pub const SPRITE_SHADER_HANDLE: u64 = 2; // TODO Create HandleUntyped once TypeId::of is const
pub const SPRITE_COLORED_SHADER_HANDLE: u64 = 3; // TODO Create HandleUntyped once TypeId::of is const

pub fn sprite_plugin(app: &mut App, render_app: &mut App) {
    let mut shaders = app.world.resource_mut::<Assets<Shader>>();
    let sprite_shader = Shader::from_wgsl(include_str!("sprite/render/sprite.wgsl"));
    let sprite_colored_shader =
        Shader::from_wgsl(include_str!("sprite/render/sprite_colored.wgsl"));
    shaders.set_untracked(
        HandleId::with_id::<Shader>(SPRITE_SHADER_HANDLE),
        sprite_shader,
    );
    shaders.set_untracked(
        HandleId::with_id::<Shader>(SPRITE_COLORED_SHADER_HANDLE),
        sprite_colored_shader,
    );
    app.add_asset::<TextureAtlas>();

    render_app
        .init_resource::<ImageBindGroups>()
        .init_resource::<SpritePipeline>()
        .init_resource::<SpecializedPipelines<SpritePipeline>>()
        .init_resource::<SpriteMeta>()
        .init_resource::<ExtractedSprites>()
        .init_resource::<SpriteAssetEvents>()
        .add_render_command::<Transparent2d, DrawSprite>()
        .add_system_to_stage(RenderStage::Extract, extract_sprites.system(&mut app.world))
        .add_system_to_stage(
            RenderStage::Extract,
            extract_sprite_events.system(&mut app.world),
        )
        .add_system_to_stage(RenderStage::Queue, queue_sprites);

    // TODO Are sprites using the sort phase and visibility plugin? Everything seems to be done in queue_sprites ignoring these.
}
