use serde::{Deserialize, Serialize};

pub const GIT_HEAD: &str = env!("GIT_HEAD");
pub const GIT_STATUS: &str = env!("GIT_STATUS");

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct GitInfo {
    pub head: String,
    pub status: String,
}

impl GitInfo {
    pub fn new() -> Self {
        Self {
            head: GIT_HEAD.to_string(),
            status: GIT_STATUS.to_string(),
        }
    }
}
