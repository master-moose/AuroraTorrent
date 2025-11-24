use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub download_path: String,
    pub max_download_speed: u64, // bytes per second
    pub max_upload_speed: u64,   // bytes per second
}

impl Default for Config {
    fn default() -> Self {
        Self {
            download_path: "downloads".to_string(),
            max_download_speed: 0, // 0 = unlimited
            max_upload_speed: 0,
        }
    }
}
