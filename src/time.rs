mod stopwatch;
mod timer;

pub use stopwatch::Stopwatch;
pub use timer::Timer;

use std::time::{Duration, Instant};

use crate::ecs::Resource;

#[derive(Debug, Clone)]
pub struct Time {
    delta: Duration,
    last_update: Option<Instant>,
    delta_seconds_f64: f64,
    delta_seconds: f32,
    seconds_since_startup: f64,
    startup: Instant,
}

impl Default for Time {
    fn default() -> Time {
        Time {
            delta: Duration::from_secs(0),
            last_update: None,
            startup: Instant::now(),
            delta_seconds_f64: 0.0,
            seconds_since_startup: 0.0,
            delta_seconds: 0.0,
        }
    }
}

impl Resource for Time {}

impl Time {
    #[inline]
    pub fn update(&mut self) {
        let now = Instant::now();
        self.update_with_instant(now);
    }

    pub(crate) fn update_with_instant(&mut self, instant: Instant) {
        if let Some(last_update) = self.last_update {
            self.delta = instant - last_update;
            self.delta_seconds_f64 = self.delta.as_secs_f64();
            self.delta_seconds = self.delta.as_secs_f32();
        }

        let duration_since_startup = instant - self.startup;
        self.seconds_since_startup = duration_since_startup.as_secs_f64();
        self.last_update = Some(instant);
    }

    #[inline]
    pub fn delta(&self) -> Duration {
        self.delta
    }

    #[inline]
    pub fn delta_seconds(&self) -> f32 {
        self.delta_seconds
    }

    #[inline]
    pub fn delta_seconds_f64(&self) -> f64 {
        self.delta_seconds_f64
    }

    #[inline]
    pub fn seconds_since_startup(&self) -> f64 {
        self.seconds_since_startup
    }

    #[inline]
    pub fn startup(&self) -> Instant {
        self.startup
    }

    #[inline]
    pub fn last_update(&self) -> Option<Instant> {
        self.last_update
    }

    #[inline]
    pub fn time_since_startup(&self) -> Duration {
        Instant::now() - self.startup
    }
}
