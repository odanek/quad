mod bundle;
mod dynamic_texture_atlas_builder;
mod mesh2d;
mod rect;
mod render;
mod sprite;
mod texture_atlas;
mod texture_atlas_builder;

pub mod collide_aabb;

pub use bundle::*;
pub use dynamic_texture_atlas_builder::*;
pub use mesh2d::*;
pub use rect::*;
pub use render::*;
pub use sprite::*;
pub use texture_atlas::*;
pub use texture_atlas_builder::*;

use crate::{
    app::{App, Stage},
    asset::{Assets, HandleUntyped},
    pipeline::Transparent2d,
    reflect::TypeUuid,
    render::render_resource::{Shader, SpecializedPipelines},
};

#[derive(Default)]
pub struct SpritePlugin;

pub const SPRITE_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 2763343953151597127);

fn sprite_plugin(app: &mut App, render_app: &mut App) {
    let mut shaders = app.world.resource_mut::<Assets<Shader>>();
    let sprite_shader = Shader::from_wgsl(include_str!("sprite/render/sprite.wgsl"));
    shaders.set_untracked(SPRITE_SHADER_HANDLE, sprite_shader);
    app.add_asset::<TextureAtlas>();
    mesh_2d_render_plugin(app, render_app);
    color_material_plugin(app, render_app);

    render_app
        .init_resource::<ImageBindGroups>()
        .init_resource::<SpritePipeline>()
        .init_resource::<SpecializedPipelines<SpritePipeline>>()
        .init_resource::<SpriteMeta>()
        .init_resource::<ExtractedSprites>()
        .init_resource::<SpriteAssetEvents>()
        .add_render_command::<Transparent2d, DrawSprite>()
        .add_system_to_stage(Stage::RenderExtract, &render::extract_sprites)
        .add_system_to_stage(Stage::RenderExtract, &render::extract_sprite_events)
        .add_system_to_stage(Stage::RenderQueue, &queue_sprites);
}
