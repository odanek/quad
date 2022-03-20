use crate::{ecs::Component, ty::Mat4};

use super::DepthCalculation;

pub trait CameraProjection {
    fn get_projection_matrix(&self) -> Mat4;
    fn update(&mut self, width: f32, height: f32);
    fn depth_calculation(&self) -> DepthCalculation;
    fn far(&self) -> f32;
}

// TODO: make this a component instead of a property
#[derive(Debug, Clone)]
pub enum WindowOrigin {
    Center,
    BottomLeft,
}

#[derive(Debug, Clone)]
pub enum ScalingMode {
    /// Manually specify left/right/top/bottom values.
    /// Ignore window resizing; the image will stretch.
    None,
    /// Match the window size. 1 world unit = 1 pixel.
    WindowSize,
    /// Keep vertical axis constant; resize horizontal with aspect ratio.
    FixedVertical,
    /// Keep horizontal axis constant; resize vertical with aspect ratio.
    FixedHorizontal,
}

#[derive(Component, Debug, Clone)]
pub struct OrthographicProjection {
    pub left: f32,
    pub right: f32,
    pub bottom: f32,
    pub top: f32,
    pub near: f32,
    pub far: f32,
    pub window_origin: WindowOrigin,
    pub scaling_mode: ScalingMode,
    pub scale: f32,
    pub depth_calculation: DepthCalculation,
}

impl CameraProjection for OrthographicProjection {
    fn get_projection_matrix(&self) -> Mat4 {
        orthographic_rh(
            self.left * self.scale,
            self.right * self.scale,
            self.bottom * self.scale,
            self.top * self.scale,
            // NOTE: near and far are swapped to invert the depth range from [0,1] to [1,0]
            // This is for interoperability with pipelines using infinite reverse perspective projections.
            self.far,
            self.near,
        )
    }

    fn update(&mut self, width: f32, height: f32) {
        match (&self.scaling_mode, &self.window_origin) {
            (ScalingMode::WindowSize, WindowOrigin::Center) => {
                let half_width = width / 2.0;
                let half_height = height / 2.0;
                self.left = -half_width;
                self.right = half_width;
                self.top = half_height;
                self.bottom = -half_height;
            }
            (ScalingMode::WindowSize, WindowOrigin::BottomLeft) => {
                self.left = 0.0;
                self.right = width;
                self.top = height;
                self.bottom = 0.0;
            }
            (ScalingMode::FixedVertical, WindowOrigin::Center) => {
                let aspect_ratio = width / height;
                self.left = -aspect_ratio;
                self.right = aspect_ratio;
                self.top = 1.0;
                self.bottom = -1.0;
            }
            (ScalingMode::FixedVertical, WindowOrigin::BottomLeft) => {
                let aspect_ratio = width / height;
                self.left = 0.0;
                self.right = aspect_ratio;
                self.top = 1.0;
                self.bottom = 0.0;
            }
            (ScalingMode::FixedHorizontal, WindowOrigin::Center) => {
                let aspect_ratio = height / width;
                self.left = -1.0;
                self.right = 1.0;
                self.top = aspect_ratio;
                self.bottom = -aspect_ratio;
            }
            (ScalingMode::FixedHorizontal, WindowOrigin::BottomLeft) => {
                let aspect_ratio = height / width;
                self.left = 0.0;
                self.right = 1.0;
                self.top = aspect_ratio;
                self.bottom = 0.0;
            }
            (ScalingMode::None, _) => {}
        }
    }

    fn depth_calculation(&self) -> DepthCalculation {
        self.depth_calculation
    }

    fn far(&self) -> f32 {
        self.far
    }
}

impl Default for OrthographicProjection {
    fn default() -> Self {
        OrthographicProjection {
            left: -1.0,
            right: 1.0,
            bottom: -1.0,
            top: 1.0,
            near: 0.0,
            far: 1000.0,
            window_origin: WindowOrigin::Center,
            scaling_mode: ScalingMode::WindowSize,
            scale: 1.0,
            depth_calculation: DepthCalculation::Distance,
        }
    }
}

fn orthographic_rh<T: cgm::Float>(
    left: T,
    right: T,
    bottom: T,
    top: T,
    near: T,
    far: T,
) -> cgm::Mat4<T> {
    let rcp_width = T::ONE / (right - left);
    let rcp_height = T::ONE / (top - bottom);
    let r = T::ONE / (near - far);
    cgm::Mat4::from_cols(
        cgm::Vec4::new(rcp_width + rcp_width, T::ZERO, T::ZERO, T::ZERO),
        cgm::Vec4::new(T::ZERO, rcp_height + rcp_height, T::ZERO, T::ZERO),
        cgm::Vec4::new(T::ZERO, T::ZERO, r, T::ZERO),
        cgm::Vec4::new(
            -(left + right) * rcp_width,
            -(top + bottom) * rcp_height,
            r * near,
            T::ONE,
        ),
    )
}
