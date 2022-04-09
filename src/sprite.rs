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
    asset::{Assets, HandleUntyped},
    ecs::IntoSystem,
    pipeline::Transparent2d,
    reflect::TypeUuid,
    render::render_resource::{Shader, SpecializedPipelines},
};

#[derive(Default)]
pub struct SpritePlugin;

pub const SPRITE_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 2763343953151597127);
pub const SPRITE_COLORED_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 2263383953151507126); // TODO What is the id good for?

pub fn sprite_plugin(app: &mut App, render_app: &mut App) {
    let mut shaders = app.world.resource_mut::<Assets<Shader>>();
    let sprite_shader = Shader::from_wgsl(include_str!("sprite/render/sprite.wgsl"));
    let sprite_colored_shader =
        Shader::from_wgsl(include_str!("sprite/render/sprite_colored.wgsl"));
    shaders.set_untracked(SPRITE_SHADER_HANDLE, sprite_shader);
    shaders.set_untracked(SPRITE_COLORED_SHADER_HANDLE, sprite_colored_shader);
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
        .add_system_to_stage(RenderStage::Queue, &queue_sprites);

    // TODO Are sprites using the sort phase and visibility plugin? Everything seems to be done in queue_sprites ignoring these.
}
