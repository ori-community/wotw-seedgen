/// Access to plandomizers
pub trait PlandoAccess {
    /// Read a plando by identifier
    fn read_plando(&self, identifier: &str) -> Result<Vec<u8>, String>;

    /// Return a `Vec` of identifiers which may be passed to [`PlandoAccess::read_plando`]
    fn available_plandos(&self) -> Vec<String>;
}
