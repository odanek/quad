mod context;
mod quad;
mod runner;
mod scene;

pub use self::quad::{Quad, QuadConfig};
pub use scene::{Scene, SceneResult, SceneStage};

pub mod prelude {
    pub use crate::run::{Quad, QuadConfig, Scene, SceneResult, SceneStage};
}
