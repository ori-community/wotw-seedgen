use log::LevelFilter;
use log4rs::{
    append::{
        console::{ConsoleAppender, Target},
        file::FileAppender,
    },
    config::{Appender, Config, Root},
    encode::{json::JsonEncoder, pattern::PatternEncoder, Encode},
    filter::threshold::ThresholdFilter,
};

pub fn initialize_log(
    use_file: Option<&str>,
    stderr_log_level: LevelFilter,
    json: bool,
) -> Result<(), String> {
    let encoder: Box<dyn Encode> = if json {
        Box::new(JsonEncoder::new())
    } else {
        Box::new(PatternEncoder::new("{h({l}):5}  {m}{n}"))
    };

    let stderr = ConsoleAppender::builder()
        .target(Target::Stderr)
        .encoder(encoder)
        .build();

    let log_config = if let Some(path) = use_file {
        let log_file = FileAppender::builder()
            .append(false)
            .encoder(Box::new(PatternEncoder::new("{l:5}  {m}{n}")))
            .build(path)
            .map_err(|err| format!("Failed to create log file: {}", err))?;

        Config::builder()
            .appender(Appender::builder().build("log_file", Box::new(log_file)))
            .appender(
                Appender::builder()
                    .filter(Box::new(ThresholdFilter::new(stderr_log_level)))
                    .build("stderr", Box::new(stderr)),
            )
            .build(
                Root::builder()
                    .appender("stderr")
                    .appender("log_file")
                    .build(LevelFilter::Trace),
            )
            .map_err(|err| format!("Failed to configure logger: {}", err))?
    } else {
        Config::builder()
            .appender(Appender::builder().build("stderr", Box::new(stderr)))
            .build(Root::builder().appender("stderr").build(stderr_log_level))
            .map_err(|err| format!("Failed to configure logger: {}", err))?
    };

    log4rs::init_config(log_config)
        .map_err(|err| format!("Failed to initialize logger: {}", err))?;
    #[cfg(target_os = "windows")]
    ansi_term::enable_ansi_support()
        .unwrap_or_else(|err| log::warn!("Failed to enable ansi support: {}", err));

    Ok(())
}
