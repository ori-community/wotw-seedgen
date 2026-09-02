pub use log::Level;
use serde::Serialize;
use utoipa::ToSchema;

use std::sync::Mutex;

use log::{LevelFilter, Log, Metadata};

// search for log uses that don't go through the capture: (trace|debug|info|warn|error|log_enabled)!\((?![\s\n]*?logger:)
// ideally also check whether log_capture refs are forwarded properly

#[derive(Debug)]
pub struct LogCapture {
    max_level: LevelFilter,
    records: Mutex<Vec<Record>>,
}

pub static NO_LOG_CAPTURE: LogCapture = LogCapture::new();

impl LogCapture {
    pub const fn new() -> Self {
        Self {
            max_level: LevelFilter::Off,
            records: Mutex::new(Vec::new()),
        }
    }

    pub fn with_max_level(mut self, level: LevelFilter) -> Self {
        self.max_level = level;
        self
    }

    pub fn finish(self) -> Vec<Record> {
        self.records.into_inner().unwrap()
    }
}

impl Default for LogCapture {
    fn default() -> Self {
        Self::new()
    }
}

impl Log for LogCapture {
    fn enabled(&self, metadata: &Metadata) -> bool {
        log::logger().enabled(metadata) || self.max_level >= metadata.level()
    }

    fn log(&self, record: &log::Record) {
        let global_logger = log::logger();

        let level = record.metadata().level();
        if self.max_level < level {
            global_logger.log(record);
            return;
        }

        let message = record.args().to_string();

        global_logger.log(
            &log::Record::builder()
                .args(format_args!("{message}"))
                .metadata(record.metadata().clone())
                .module_path(record.module_path())
                .file(record.file())
                .line(record.line())
                .build(),
        );

        self.records
            .lock()
            .unwrap()
            .push(Record::new(level, message));
    }

    fn flush(&self) {
        log::logger().flush();
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct Record {
    #[schema(value_type = LevelSchema)]
    pub level: Level,
    pub message: String,
}

impl Record {
    fn new(level: Level, message: String) -> Self {
        Self { level, message }
    }
}

#[derive(ToSchema)]
#[serde(rename_all = "UPPERCASE")]
#[allow(unused)]
enum LevelSchema {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}
