use crate::{
    ecs::{Res, SystemParamItem},
    render::{
        render_asset::{PrepareAssetError, RenderAsset},
        render_resource::{Sampler, Texture, TextureView},
        renderer::{RenderDevice, RenderQueue},
    },
    ty::Vec2,
};

use super::{TEXTURE_FORMAT, image_texture_conversion::image_to_texture};
use thiserror::Error;
use wgpu::{
    Extent3d, ImageCopyTexture, ImageDataLayout, Origin3d, TextureDimension, TextureFormat,
    TextureViewDescriptor,
};

pub const TEXTURE_ASSET_INDEX: u64 = 0;
pub const SAMPLER_ASSET_INDEX: u64 = 1;
pub const DEFAULT_IMAGE_HANDLE: u64 = 1; // TODO Create HandleUntyped once TypeId::of is const

#[derive(Debug)]
pub enum ImageFormat {
    Avif,
    Basis,
    Bmp,
    Dds,
    Farbfeld,
    Gif,
    Hdr,
    Ico,
    Jpeg,
    Ktx2,
    Png,
    Pnm,
    Tga,
    Tiff,
    WebP,
}

impl ImageFormat {
    pub fn from_mime_type(mime_type: &str) -> Option<Self> {
        Some(match mime_type.to_ascii_lowercase().as_str() {
            "image/bmp" | "image/x-bmp" => ImageFormat::Bmp,
            "image/vnd-ms.dds" => ImageFormat::Dds,
            "image/jpeg" => ImageFormat::Jpeg,
            "image/ktx2" => ImageFormat::Ktx2,
            "image/png" => ImageFormat::Png,
            "image/x-targa" | "image/x-tga" => ImageFormat::Tga,
            _ => return None,
        })
    }

    pub fn from_extension(extension: &str) -> Option<Self> {
        Some(match extension.to_ascii_lowercase().as_str() {
            "avif" => ImageFormat::Avif,
            "basis" => ImageFormat::Basis,
            "bmp" => ImageFormat::Bmp,
            "dds" => ImageFormat::Dds,
            "ff" | "farbfeld" => ImageFormat::Farbfeld,
            "gif" => ImageFormat::Gif,
            "hdr" => ImageFormat::Hdr,
            "ico" => ImageFormat::Ico,
            "jpg" | "jpeg" => ImageFormat::Jpeg,
            "ktx2" => ImageFormat::Ktx2,
            "pbm" | "pam" | "ppm" | "pgm" => ImageFormat::Pnm,
            "png" => ImageFormat::Png,
            "tga" => ImageFormat::Tga,
            "tif" | "tiff" => ImageFormat::Tiff,
            "webp" => ImageFormat::WebP,
            _ => return None,
        })
    }

    pub fn as_image_crate_format(&self) -> Option<image::ImageFormat> {
        Some(match self {
            ImageFormat::Avif => image::ImageFormat::Avif,
            ImageFormat::Bmp => image::ImageFormat::Bmp,
            ImageFormat::Dds => image::ImageFormat::Dds,
            ImageFormat::Farbfeld => image::ImageFormat::Farbfeld,
            ImageFormat::Gif => image::ImageFormat::Gif,
            ImageFormat::Hdr => image::ImageFormat::Hdr,
            ImageFormat::Ico => image::ImageFormat::Ico,
            ImageFormat::Jpeg => image::ImageFormat::Jpeg,
            ImageFormat::Png => image::ImageFormat::Png,
            ImageFormat::Pnm => image::ImageFormat::Pnm,
            ImageFormat::Tga => image::ImageFormat::Tga,
            ImageFormat::Tiff => image::ImageFormat::Tiff,
            ImageFormat::WebP => image::ImageFormat::WebP,
            ImageFormat::Basis | ImageFormat::Ktx2 => return None,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Image {
    pub data: Vec<u8>,
    // TODO: this nesting makes accessing Image metadata verbose. Either flatten out descriptor or add accessors
    pub texture_descriptor: wgpu::TextureDescriptor<'static>,
    pub sampler_descriptor: wgpu::SamplerDescriptor<'static>,
}

impl Default for Image {
    fn default() -> Self {
        let format = TEXTURE_FORMAT;
        let data = vec![255; format.pixel_size()];
        Image {
            data,
            texture_descriptor: wgpu::TextureDescriptor {
                size: wgpu::Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                format,
                dimension: wgpu::TextureDimension::D2,
                label: None,
                mip_level_count: 1,
                sample_count: 1,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            },
            sampler_descriptor: wgpu::SamplerDescriptor::default(),
        }
    }
}

impl Image {
    /// Creates a new image from raw binary data and the corresponding metadata.
    ///
    /// # Panics
    /// Panics if the length of the `data`, volume of the `size` and the size of the `format`
    /// do not match.
    pub fn new(
        size: Extent3d,
        dimension: TextureDimension,
        data: Vec<u8>,
        format: TextureFormat,
    ) -> Self {
        debug_assert_eq!(
            size.volume() * format.pixel_size(),
            data.len(),
            "Pixel data, size and format have to match",
        );
        let mut image = Self {
            data,
            ..Default::default()
        };
        image.texture_descriptor.dimension = dimension;
        image.texture_descriptor.size = size;
        image.texture_descriptor.format = format;
        image
    }

    /// Creates a new image from raw binary data and the corresponding metadata, by filling
    /// the image data with the `pixel` data repeated multiple times.
    ///
    /// # Panics
    /// Panics if the size of the `format` is not a multiple of the length of the `pixel` data.
    /// do not match.
    pub fn new_fill(
        size: Extent3d,
        dimension: TextureDimension,
        pixel: &[u8],
        format: TextureFormat,
    ) -> Self {
        let mut value = Image::default();
        value.texture_descriptor.format = format;
        value.texture_descriptor.dimension = dimension;
        value.resize(size);

        debug_assert_eq!(
            pixel.len() % format.pixel_size(),
            0,
            "Must not have incomplete pixel data."
        );
        debug_assert!(
            pixel.len() <= value.data.len(),
            "Fill data must fit within pixel buffer."
        );

        for current_pixel in value.data.chunks_exact_mut(pixel.len()) {
            current_pixel.copy_from_slice(pixel);
        }
        value
    }

    /// Returns the aspect ratio (height/width) of a 2D image.
    pub fn aspect_2d(&self) -> f32 {
        self.texture_descriptor.size.height as f32 / self.texture_descriptor.size.width as f32
    }

    /// Returns the size of a 2D image.
    pub fn size(&self) -> Vec2 {
        Vec2::new(
            self.texture_descriptor.size.width as f32,
            self.texture_descriptor.size.height as f32,
        )
    }

    /// Resizes the image to the new size, by removing information or appending 0 to the `data`.
    /// Does not properly resize the contents of the image, but only its internal `data` buffer.
    pub fn resize(&mut self, size: Extent3d) {
        self.texture_descriptor.size = size;
        self.data.resize(
            size.volume() * self.texture_descriptor.format.pixel_size(),
            0,
        );
    }

    /// Changes the `size`, asserting that the total number of data elements (pixels) remains the
    /// same.
    ///
    /// # Panics
    /// Panics if the `new_size` does not have the same volume as to old one.
    pub fn reinterpret_size(&mut self, new_size: Extent3d) {
        assert!(
            new_size.volume() == self.texture_descriptor.size.volume(),
            "Incompatible sizes: old = {:?} new = {:?}",
            self.texture_descriptor.size,
            new_size
        );

        self.texture_descriptor.size = new_size;
    }

    /// Takes a 2D image containing vertically stacked images of the same size, and reinterprets
    /// it as a 2D array texture, where each of the stacked images becomes one layer of the
    /// array. This is primarily for use with the `texture2DArray` shader uniform type.
    ///
    /// # Panics
    /// Panics if the texture is not 2D, has more than one layers or is not evenly dividable into
    /// the `layers`.
    pub fn reinterpret_stacked_2d_as_array(&mut self, layers: u32) {
        // Must be a stacked image, and the height must be divisible by layers.
        assert!(self.texture_descriptor.dimension == TextureDimension::D2);
        assert!(self.texture_descriptor.size.depth_or_array_layers == 1);
        assert_eq!(self.texture_descriptor.size.height % layers, 0);

        self.reinterpret_size(Extent3d {
            width: self.texture_descriptor.size.width,
            height: self.texture_descriptor.size.height / layers,
            depth_or_array_layers: layers,
        });
    }

    /// Convert a texture from a format to another
    /// Only a few formats are supported as input and output:
    /// - `TextureFormat::R8Unorm`
    /// - `TextureFormat::Rg8Unorm`
    /// - `TextureFormat::Rgba8UnormSrgb`
    /// - `TextureFormat::Bgra8UnormSrgb`
    pub fn convert(&self, new_format: TextureFormat) -> Option<Self> {
        super::image_texture_conversion::texture_to_image(self)
            .and_then(|img| match new_format {
                TextureFormat::R8Unorm => {
                    Some((image::DynamicImage::ImageLuma8(img.into_luma8()), false))
                }
                TextureFormat::Rg8Unorm => Some((
                    image::DynamicImage::ImageLumaA8(img.into_luma_alpha8()),
                    false,
                )),
                TextureFormat::Rgba8UnormSrgb => {
                    Some((image::DynamicImage::ImageRgba8(img.into_rgba8()), true))
                }
                _ => None,
            })
            .map(|(dyn_img, is_srgb)| {
                super::image_texture_conversion::image_to_texture(dyn_img, is_srgb)
            })
    }

    /// Load a bytes buffer in a [`Image`], according to type `image_type`, using the `image`
    /// crate
    pub fn from_buffer(
        buffer: &[u8],
        image_type: ImageType,
        is_srgb: bool,
    ) -> Result<Image, TextureError> {
        let format = image_type.to_image_format()?;

        // Load the image in the expected format.
        // Some formats like PNG allow for R or RG textures too, so the texture
        // format needs to be determined. For RGB textures an alpha channel
        // needs to be added, so the image data needs to be converted in those
        // cases.

        let image_crate_format = format
            .as_image_crate_format()
            .ok_or_else(|| TextureError::UnsupportedTextureFormat(format!("{format:?}")))?;
        let dyn_img = image::load_from_memory_with_format(buffer, image_crate_format)?;
        Ok(image_to_texture(dyn_img, is_srgb))
    }
}

/// An error that occurs when loading a texture
#[derive(Error, Debug)]
pub enum TextureError {
    #[error("invalid image mime type")]
    InvalidImageMimeType(String),
    #[error("invalid image extension")]
    InvalidImageExtension(String),
    #[error("failed to load an image: {0}")]
    ImageError(#[from] image::ImageError),
    #[error("unsupported texture format: {0}")]
    UnsupportedTextureFormat(String),
}

/// The type of a raw image buffer.
pub enum ImageType<'a> {
    /// The mime type of an image, for example `"image/png"`.
    MimeType(&'a str),
    /// The extension of an image file, for example `"png"`.
    Extension(&'a str),
}

impl<'a> ImageType<'a> {
    pub fn to_image_format(&self) -> Result<ImageFormat, TextureError> {
        match self {
            ImageType::MimeType(mime_type) => ImageFormat::from_mime_type(mime_type)
                .ok_or_else(|| TextureError::InvalidImageMimeType(mime_type.to_string())),
            ImageType::Extension(extension) => ImageFormat::from_extension(extension)
                .ok_or_else(|| TextureError::InvalidImageExtension(extension.to_string())),
        }
    }
}

/// Used to calculate the volume of an item.
pub trait Volume {
    fn volume(&self) -> usize;
}

impl Volume for Extent3d {
    /// Calculates the volume of the [`Extent3d`].
    fn volume(&self) -> usize {
        (self.width * self.height * self.depth_or_array_layers) as usize
    }
}

/// Information about the pixel size in bytes and the number of different components.
pub struct PixelInfo {
    /// The size of a component of a pixel in bytes.
    pub type_size: usize,
    /// The amount of different components (color channels).
    pub num_components: usize,
}

/// Extends the wgpu [`TextureFormat`] with information about the pixel.
pub trait TextureFormatPixelInfo {
    /// Returns the pixel information of the format.
    fn pixel_info(&self) -> PixelInfo;
    /// Returns the size of a pixel of the format.
    fn pixel_size(&self) -> usize {
        let info = self.pixel_info();
        info.type_size * info.num_components
    }
}

impl TextureFormatPixelInfo for TextureFormat {
    fn pixel_info(&self) -> PixelInfo {
        let type_size = match self {
            // 8bit
            TextureFormat::R8Unorm
            | TextureFormat::R8Snorm
            | TextureFormat::R8Uint
            | TextureFormat::R8Sint
            | TextureFormat::Rg8Unorm
            | TextureFormat::Rg8Snorm
            | TextureFormat::Rg8Uint
            | TextureFormat::Rg8Sint
            | TextureFormat::Rgba8Unorm
            | TextureFormat::Rgba8UnormSrgb
            | TextureFormat::Rgba8Snorm
            | TextureFormat::Rgba8Uint
            | TextureFormat::Rgba8Sint
            | TextureFormat::Bgra8Unorm
            | TextureFormat::Bgra8UnormSrgb => 1,

            // 16bit
            TextureFormat::R16Uint
            | TextureFormat::R16Sint
            | TextureFormat::R16Float
            | TextureFormat::Rg16Uint
            | TextureFormat::Rg16Sint
            | TextureFormat::Rg16Float
            | TextureFormat::Rgba16Uint
            | TextureFormat::Rgba16Sint
            | TextureFormat::Rgba16Float => 2,

            // 32bit
            TextureFormat::R32Uint
            | TextureFormat::R32Sint
            | TextureFormat::R32Float
            | TextureFormat::Rg32Uint
            | TextureFormat::Rg32Sint
            | TextureFormat::Rg32Float
            | TextureFormat::Rgba32Uint
            | TextureFormat::Rgba32Sint
            | TextureFormat::Rgba32Float
            | TextureFormat::Depth32Float => 4,

            // special cases
            TextureFormat::Rgb10a2Unorm => 4,
            TextureFormat::Rg11b10Float => 4,
            TextureFormat::Depth24Plus => 3, // FIXME is this correct?
            TextureFormat::Depth24PlusStencil8 => 4,
            // TODO: this is not good! this is a temporary step while porting bevy_render to direct wgpu usage
            _ => panic!("cannot get pixel info for type"),
        };

        let components = match self {
            TextureFormat::R8Unorm
            | TextureFormat::R8Snorm
            | TextureFormat::R8Uint
            | TextureFormat::R8Sint
            | TextureFormat::R16Uint
            | TextureFormat::R16Sint
            | TextureFormat::R16Float
            | TextureFormat::R32Uint
            | TextureFormat::R32Sint
            | TextureFormat::R32Float => 1,

            TextureFormat::Rg8Unorm
            | TextureFormat::Rg8Snorm
            | TextureFormat::Rg8Uint
            | TextureFormat::Rg8Sint
            | TextureFormat::Rg16Uint
            | TextureFormat::Rg16Sint
            | TextureFormat::Rg16Float
            | TextureFormat::Rg32Uint
            | TextureFormat::Rg32Sint
            | TextureFormat::Rg32Float => 2,

            TextureFormat::Rgba8Unorm
            | TextureFormat::Rgba8UnormSrgb
            | TextureFormat::Rgba8Snorm
            | TextureFormat::Rgba8Uint
            | TextureFormat::Rgba8Sint
            | TextureFormat::Bgra8Unorm
            | TextureFormat::Bgra8UnormSrgb
            | TextureFormat::Rgba16Uint
            | TextureFormat::Rgba16Sint
            | TextureFormat::Rgba16Float
            | TextureFormat::Rgba32Uint
            | TextureFormat::Rgba32Sint
            | TextureFormat::Rgba32Float => 4,

            // special cases
            TextureFormat::Rgb10a2Unorm
            | TextureFormat::Rg11b10Float
            | TextureFormat::Depth32Float
            | TextureFormat::Depth24Plus
            | TextureFormat::Depth24PlusStencil8 => 1,
            // TODO: this is not good! this is a temporary step while porting bevy_render to direct wgpu usage
            _ => panic!("cannot get pixel info for type"),
        };

        PixelInfo {
            type_size,
            num_components: components,
        }
    }
}

/// The GPU-representation of an [`Image`].
/// Consists of the [`Texture`], its [`TextureView`] and the corresponding [`Sampler`], and the texture's [`Size`].
#[derive(Debug, Clone)]
pub struct GpuImage {
    pub texture: Texture,
    pub texture_view: TextureView,
    pub texture_format: TextureFormat,
    pub sampler: Sampler,
    pub size: Vec2,
}

impl RenderAsset for Image {
    type ExtractedAsset = Image;
    type PreparedAsset = GpuImage;
    type Param = (Res<'static, RenderDevice>, Res<'static, RenderQueue>);

    /// Clones the Image.
    fn extract_asset(&self) -> Self::ExtractedAsset {
        self.clone()
    }

    /// Converts the extracted image into a [`GpuImage`].
    fn prepare_asset(
        image: Self::ExtractedAsset,
        (render_device, render_queue): &mut SystemParamItem<Self::Param>,
    ) -> Result<Self::PreparedAsset, PrepareAssetError<Self::ExtractedAsset>> {
        let texture = render_device.create_texture(&image.texture_descriptor);
        let sampler = render_device.create_sampler(&image.sampler_descriptor);

        let format_size = image.texture_descriptor.format.pixel_size();
        render_queue.write_texture(
            ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &image.data,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(image.texture_descriptor.size.width * format_size as u32),
                rows_per_image: if image.texture_descriptor.size.depth_or_array_layers > 1 {
                    Some(image.texture_descriptor.size.height)
                } else {
                    None
                },
            },
            image.texture_descriptor.size,
        );

        let texture_view = texture.create_view(&TextureViewDescriptor::default());
        let size = Vec2::new(
            image.texture_descriptor.size.width as f32,
            image.texture_descriptor.size.height as f32,
        );
        Ok(GpuImage {
            texture,
            texture_view,
            texture_format: image.texture_descriptor.format,
            sampler,
            size,
        })
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn image_size() {
        let size = Extent3d {
            width: 200,
            height: 100,
            depth_or_array_layers: 1,
        };
        let image = Image::new_fill(
            size,
            TextureDimension::D2,
            &[0, 0, 0, 255],
            TextureFormat::Rgba8Unorm,
        );
        assert_eq!(
            Vec2::new(size.width as f32, size.height as f32),
            image.size()
        );
    }
    #[test]
    fn image_default_size() {
        let image = Image::default();
        assert_eq!(Vec2::new(1.0, 1.0), image.size());
    }
}
