use std::sync::Once;

use env_logger::{Builder, Env};

use crate::assets::TEST_ASSETS;

static LOGGER_INITIALIZED: Once = Once::new();

pub fn test_logger() {
    LOGGER_INITIALIZED.call_once(|| {
        // Avoid log spam from test asset initialization
        let _ = *TEST_ASSETS;

        Builder::from_env(Env::default().default_filter_or("debug"))
            .format_timestamp(None)
            .is_test(true)
            .init();
    });
}
