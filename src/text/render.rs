use cgm::Zero;

use crate::{
    asset::Assets,
    ecs::{Bundle, Changed, Entity, Local, ParamSet, Query, Res, ResMut, With},
    render::{
        extract_param::Extract,
        texture::Image,
        view::{ComputedVisibility, Visibility},
    },
    sprite::{Anchor, ExtractedSprite, ExtractedSprites, TextureAtlas},
    transform::{GlobalTransform, Transform},
    ty::{Vec2, Vec3},
    windowing::{WindowId, Windows},
};

use super::{
    DefaultTextPipeline, Font, FontAtlasSet, HorizontalAlign, Text, TextError, TextSize,
    VerticalAlign,
};

/// The bundle of components needed to draw text in a 2D scene via a 2D `OrthographicCameraBundle`.
/// [Example usage.](https://github.com/bevyengine/bevy/blob/latest/examples/2d/text2d.rs)
#[derive(Bundle, Clone, Debug, Default)]
pub struct TextBundle {
    pub text: Text,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub text_size: TextSize,
    pub visibility: Visibility,
    pub computed_visibility: ComputedVisibility,
}

#[allow(clippy::type_complexity)]
pub fn extract_text_sprite(
    mut extracted_sprites: ResMut<ExtractedSprites>,
    texture_atlases: Extract<Res<Assets<TextureAtlas>>>,
    text_pipeline: Extract<Res<DefaultTextPipeline>>,
    windows: Extract<Res<Windows>>,
    text2d_query: Extract<
        Query<(
            Entity,
            &ComputedVisibility,
            &Text,
            &GlobalTransform,
            &TextSize,
        )>,
    >,
) {
    let scale_factor = windows.scale_factor(WindowId::primary()) as f32;

    for (entity, visibility, text, transform, calculated_size) in text2d_query.iter() {
        if !visibility.is_visible() {
            continue;
        }
        let (width, height) = (calculated_size.size.x, calculated_size.size.y);

        if let Some(text_layout) = text_pipeline.get_glyphs(&entity) {
            let text_glyphs = &text_layout.glyphs;
            let alignment_offset = match text.alignment.vertical {
                VerticalAlign::Top => Vec3::new(0.0, -height, 0.0),
                VerticalAlign::Center => Vec3::new(0.0, -height * 0.5, 0.0),
                VerticalAlign::Bottom => Vec3::ZERO,
            } + match text.alignment.horizontal {
                HorizontalAlign::Left => Vec3::ZERO,
                HorizontalAlign::Center => Vec3::new(-width * 0.5, 0.0, 0.0),
                HorizontalAlign::Right => Vec3::new(-width, 0.0, 0.0),
            };

            let mut text_transform = *transform;
            text_transform.scale /= scale_factor;

            for text_glyph in text_glyphs {
                let color = text.sections[text_glyph.section_index]
                    .style
                    .color
                    .as_rgba_linear();
                let atlas = texture_atlases
                    .get(text_glyph.atlas_info.texture_atlas.clone_weak())
                    .unwrap();
                let handle = atlas.texture.clone_weak();
                let index = text_glyph.atlas_info.glyph_index;
                let rect = Some(atlas.textures[index]);

                let glyph_transform = Transform::from_translation(
                    alignment_offset * scale_factor + text_glyph.position.extend(0.),
                );

                let transform = text_transform.mul_transform(glyph_transform);

                extracted_sprites.sprites.push(ExtractedSprite {
                    transform,
                    color,
                    rect,
                    custom_size: None,
                    image_handle_id: handle.id,
                    flip_x: false,
                    flip_y: false,
                    anchor: Anchor::Center.as_vec(),
                });
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct QueuedText {
    entities: Vec<Entity>,
}

/// Updates the layout and size information whenever the text or style is changed.
/// This information is computed by the `TextPipeline` on insertion, then stored.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn text_system(
    mut queued_text: Local<QueuedText>,
    mut textures: ResMut<Assets<Image>>,
    fonts: Res<Assets<Font>>,
    windows: Res<Windows>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut font_atlas_set_storage: ResMut<Assets<FontAtlasSet>>,
    mut text_pipeline: ResMut<DefaultTextPipeline>,
    mut text_queries: ParamSet<(
        Query<Entity, (With<TextSize>, Changed<Text>)>,
        Query<(&'static Text, &'static mut TextSize), With<TextSize>>,
    )>,
) {
    // Adds all entities where the text or the style has changed to the local queue
    for entity in text_queries.p0().iter_mut() {
        queued_text.entities.push(entity);
    }

    if queued_text.entities.is_empty() {
        return;
    }

    let scale_factor = windows.scale_factor(WindowId::primary());

    // Computes all text in the local queue
    let mut new_queue = Vec::new();
    let mut query = text_queries.p1();
    for entity in queued_text.entities.drain(..) {
        if let Ok((text, mut calculated_size)) = query.get_mut(entity) {
            match text_pipeline.queue_text(
                entity,
                &fonts,
                &text.sections,
                scale_factor,
                text.alignment,
                Vec2::new(f32::MAX, f32::MAX),
                &mut font_atlas_set_storage,
                &mut texture_atlases,
                &mut textures,
            ) {
                Err(TextError::NoSuchFont) => {
                    // There was an error processing the text layout, let's add this entity to the
                    // queue for further processing
                    new_queue.push(entity);
                }
                Err(e @ TextError::FailedToAddGlyph(_)) => {
                    panic!("Fatal error when processing text: {e}.");
                }
                Ok(size) => {
                    calculated_size.size = Vec2::new(
                        scale_value(size.x, 1. / scale_factor),
                        scale_value(size.y, 1. / scale_factor),
                    );
                }
            }
        }
    }

    queued_text.entities = new_queue;
}

pub fn scale_value(value: f32, factor: f64) -> f32 {
    (value as f64 * factor) as f32
}
