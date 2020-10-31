use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct BWConfig {
    pub log_level: String,
    pub map: String,
}
