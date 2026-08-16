use serde::Serialize;
use utoipa::ToSchema;

/// Origin of an asset
#[derive(Clone, Serialize, ToSchema)]
pub enum AssetOrigin {
    /// This asset was found within the seedgen executable's parent directory
    ExecutableDir,
    /// This asset was found within the user data directory
    UserDataDir(String),
}
