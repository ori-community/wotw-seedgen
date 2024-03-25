#[cfg(feature = "loc_data")]
mod loc_data;
#[cfg(feature = "loc_data")]
pub use loc_data::LOC_DATA;
#[cfg(feature = "presets")]
mod presets;
#[cfg(feature = "presets")]
pub use presets::{StaticPresetAccess, PRESET_ACCESS};
#[cfg(feature = "snippets")]
mod snippets;
#[cfg(feature = "snippets")]
pub use snippets::{StaticSnippetAccess, SNIPPET_ACCESS};
#[cfg(feature = "state_data")]
mod state_data;
#[cfg(feature = "state_data")]
pub use state_data::STATE_DATA;
#[cfg(feature = "uber_state_data")]
mod uber_state_data;
#[cfg(feature = "uber_state_data")]
pub use uber_state_data::UBER_STATE_DATA;

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "loc_data")]
    #[test]
    fn loc_data() {
        let _ = &*LOC_DATA;
    }

    #[cfg(feature = "state_data")]
    #[test]
    fn state_data() {
        let _ = &*STATE_DATA;
    }

    #[cfg(feature = "uber_state_data")]
    #[test]
    fn uber_state_data() {
        let _ = &*UBER_STATE_DATA;
    }

    #[cfg(feature = "snippets")]
    #[test]
    fn snippets() {
        let _ = &*SNIPPET_ACCESS;
    }

    #[cfg(feature = "presets")]
    #[test]
    fn presets() {
        let _ = &*PRESET_ACCESS;
    }
}
