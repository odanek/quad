mod stopwatch;
mod time;
mod timer;

pub use stopwatch::Stopwatch;
pub use time::Time;
pub use timer::Timer;

use crate::app::App;

pub mod prelude {
    pub use crate::timing::{Stopwatch, Time, Timer};
}

pub fn timing_plugin(app: &mut App) {
    app.init_resource::<Time>();
}
