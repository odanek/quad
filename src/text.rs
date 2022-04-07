mod error;
mod font;
mod font_atlas;
mod font_atlas_set;
mod font_loader;
mod glyph_brush;
mod pipeline;
mod render;
#[allow(clippy::module_inception)]
mod text;

use derive_deref::{Deref, DerefMut};
pub use error::*;
pub use font::*;
pub use font_atlas::*;
pub use font_atlas_set::*;
pub use font_loader::*;
pub use glyph_brush::*;
pub use pipeline::*;
pub use render::*;
pub use text::*;

use crate::{
    app::{App, Stage},
    ecs::{Entity, IntoSystem, Resource},
};

#[derive(Default, Resource, Deref, DerefMut)]
pub struct DefaultTextPipeline(TextPipeline<Entity>);

pub fn text_plugin(app: &mut App, render_app: &mut App) {
    app.add_asset::<Font>()
        .add_asset::<FontAtlasSet>()
        .init_asset_loader::<FontLoader>()
        .insert_resource(DefaultTextPipeline::default())
        .add_system_to_stage(Stage::PostUpdate, &text_system);

    render_app.add_system_to_stage(
        Stage::RenderExtract,
        extract_text_sprite.system(&mut app.world), // Must come after extract_sprites
    );
}
