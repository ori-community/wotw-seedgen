mod loc_data;
pub use loc_data::LOC_DATA;
mod presets;
pub use presets::{StaticPresetAccess, PRESET_ACCESS};
mod snippets;
pub use snippets::{StaticSnippetAccess, SNIPPET_ACCESS};
mod state_data;
pub use state_data::STATE_DATA;
mod uber_state_data;
pub use uber_state_data::UBER_STATE_DATA;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loc_data() {
        let _ = &*LOC_DATA;
    }

    #[test]
    fn state_data() {
        let _ = &*STATE_DATA;
    }

    #[test]
    fn uber_state_data() {
        let _ = &*UBER_STATE_DATA;
    }

    #[test]
    fn snippets() {
        let _ = &*SNIPPET_ACCESS;
    }

    #[test]
    fn presets() {
        let _ = &*PRESET_ACCESS;
    }
}
