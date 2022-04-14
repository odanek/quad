extern crate self as quad; // So that proc marcros can import stuff as ::quad::xxx

mod macros;

pub mod app;
pub mod asset;
pub mod audio;
pub mod ecs;
pub mod input;
pub mod pipeline;
pub mod reflect;
pub mod render;
mod run;
pub mod sprite;
pub mod tasks;
pub mod text;
pub mod timing;
pub mod transform;
pub mod ty;
pub mod windowing;

pub use run::{Quad, QuadConfig, Scene, SceneResult};
