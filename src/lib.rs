#![allow(unsafe_op_in_unsafe_fn, clippy::needless_lifetimes)]

extern crate self as quad; // So that proc marcros can import stuff as ::quad::xxx

mod macros;

pub mod app;
pub mod asset;
pub mod audio;
pub mod ecs;
pub mod input;
pub mod logging;
pub mod pipeline;
pub mod render;
pub mod run;
pub mod sprite;
pub mod tasks;
pub mod text;
pub mod timing;
pub mod transform;
pub mod ty;
pub mod ui;
pub mod windowing;

#[allow(unused_imports)]
pub mod prelude {
    pub use crate::{
        app::prelude::*, asset::prelude::*, audio::prelude::*, ecs::prelude::*, input::prelude::*,
        pipeline::prelude::*, render::prelude::*, run::prelude::*, sprite::prelude::*,
        tasks::prelude::*, text::prelude::*, timing::prelude::*, transform::prelude::*,
        ty::prelude::*, ui::prelude::*, windowing::prelude::*,
    };
}
