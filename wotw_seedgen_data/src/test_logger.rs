use std::sync::Once;

use env_logger::{Builder, Env};

static LOGGER_INITIALIZED: Once = Once::new();

pub fn test_logger() {
    LOGGER_INITIALIZED.call_once(|| {
        Builder::from_env(Env::default().default_filter_or("debug"))
            .format_timestamp(None)
            .is_test(true)
            .init();
    });
}
