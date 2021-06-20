mod core;
pub mod data;
pub mod ecs;
pub mod input;
pub mod window;

pub use crate::core::{Quad, Scene, SceneResult};
pub use ecs::{Res, ResMut, IntoSystem, System};
