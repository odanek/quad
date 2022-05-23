use log::{LevelFilter, Log, Metadata, Record};

#[cfg(debug_assertions)]
const DEFAULT_LOG_LEVEL: LevelFilter = LevelFilter::Warn;

#[cfg(not(debug_assertions))]
const DEFAULT_LOG_LEVEL: LevelFilter = LevelFilter::Error;

pub struct Logger;

impl Log for Logger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

static LOGGER: Logger = Logger;

pub fn init_logging(level: Option<LevelFilter>) {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(level.unwrap_or(DEFAULT_LOG_LEVEL)))
        .expect("Unable to initialize logging");
}
