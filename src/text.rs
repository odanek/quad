mod error;
mod font;
mod font_atlas;
mod font_atlas_set;
mod font_loader;
mod glyph_brush;
mod pipeline;
#[allow(clippy::module_inception)]
mod text;
mod text2d;

use derive_deref::{Deref, DerefMut};
pub use error::*;
pub use font::*;
pub use font_atlas::*;
pub use font_atlas_set::*;
pub use font_loader::*;
pub use glyph_brush::*;
pub use pipeline::*;
pub use text::*;
pub use text2d::*;

use crate::{
    app::{App, Stage},
    ecs::{Entity, Resource},
};

#[derive(Default, Resource, Deref, DerefMut)]
pub struct DefaultTextPipeline(TextPipeline<Entity>);

pub fn text_plugin(app: &mut App, render_app: &mut App) {
    app.add_asset::<Font>()
        .add_asset::<FontAtlasSet>()
        .init_asset_loader::<FontLoader>()
        .insert_resource(DefaultTextPipeline::default())
        .add_system_to_stage(Stage::PostUpdate, &text2d_system);

    render_app.add_system_to_stage(
        Stage::RenderExtract,
        &extract_text2d_sprite, // Must come after extract_sprites
    );
}
