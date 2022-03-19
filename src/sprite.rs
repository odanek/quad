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

// #[derive(Default)]
// pub struct SpritePlugin;

// pub const SPRITE_SHADER_HANDLE: HandleUntyped =
//     HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 2763343953151597127);

// #[derive(Debug, Hash, PartialEq, Eq, Clone, SystemLabel)]
// pub enum SpriteSystem {
//     ExtractSprites,
// }

// impl Plugin for SpritePlugin {
//     fn build(&self, app: &mut App) {
//         let mut shaders = app.world.resource_mut::<Assets<Shader>>();
//         let sprite_shader = Shader::from_wgsl(include_str!("render/sprite.wgsl"));
//         shaders.set_untracked(SPRITE_SHADER_HANDLE, sprite_shader);
//         app.add_asset::<TextureAtlas>()
//             .register_type::<Sprite>()
//             .add_plugin(Mesh2dRenderPlugin)
//             .add_plugin(ColorMaterialPlugin);

//         if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
//             render_app
//                 .init_resource::<ImageBindGroups>()
//                 .init_resource::<SpritePipeline>()
//                 .init_resource::<SpecializedPipelines<SpritePipeline>>()
//                 .init_resource::<SpriteMeta>()
//                 .init_resource::<ExtractedSprites>()
//                 .init_resource::<SpriteAssetEvents>()
//                 .add_render_command::<Transparent2d, DrawSprite>()
//                 .add_system_to_stage(
//                     RenderStage::Extract,
//                     render::extract_sprites.label(SpriteSystem::ExtractSprites),
//                 )
//                 .add_system_to_stage(RenderStage::Extract, render::extract_sprite_events)
//                 .add_system_to_stage(RenderStage::Queue, queue_sprites);
//         };
//     }
// }
