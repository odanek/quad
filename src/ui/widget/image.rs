use crate::{
    asset::Assets,
    ecs::{Component, Query, Res, With},
    render::texture::Image,
    ui::{CalculatedSize, Size, UiImage, Val},
};

/// Describes how to resize the Image node
#[derive(Component, Debug, Clone)]
pub enum ImageMode {
    /// Keep the aspect ratio of the image
    KeepAspect,
}

impl Default for ImageMode {
    fn default() -> Self {
        ImageMode::KeepAspect
    }
}

/// Updates calculated size of the node based on the image provided
pub fn image_node_system(
    textures: Res<Assets<Image>>,
    mut query: Query<(&mut CalculatedSize, &UiImage), With<ImageMode>>,
) {
    for (mut calculated_size, image) in query.iter_mut() {
        if let Some(texture) = textures.get(image.0.clone_weak()) {
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
