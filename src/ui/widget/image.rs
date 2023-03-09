use crate::{
    asset::Assets,
    ecs::{Component, Query, Res, Without},
    render::texture::Image,
    text::Text,
    ui::{CalculatedSize, Size, UiImage, Val},
};

/// Describes how to resize the Image node
#[derive(Component, Debug, Clone, Default)]
pub enum ImageMode {
    /// Keep the aspect ratio of the image
    #[default]
    KeepAspect,
}

/// Updates calculated size of the node based on the image provided
pub fn update_image_calculated_size_system(
    textures: Res<Assets<Image>>,
    mut query: Query<(&mut CalculatedSize, &UiImage), Without<Text>>,
) {
    for (mut calculated_size, image) in &mut query {
        if let Some(texture) = textures.get(&image.texture) {
            let size = Size {
                width: Val::Px(texture.texture_descriptor.size.width as f32),
                height: Val::Px(texture.texture_descriptor.size.height as f32),
            };
            // Update only if size has changed to avoid needless layout calculations
            if size != calculated_size.size {
                calculated_size.size = size;
            }
        }
    }
}
