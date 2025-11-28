use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub download_path: String,
    pub max_download_speed: u64, // bytes per second, 0 = unlimited
    pub max_upload_speed: u64,   // bytes per second, 0 = unlimited
}

impl Default for Config {
    fn default() -> Self {
        // Use user's Downloads folder as default
        let download_path = dirs::download_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap_or_default().join("Downloads"))
            .join("AuroraTorrent")
            .to_string_lossy()
            .to_string();

        Self {
            download_path,
            max_download_speed: 0,
            max_upload_speed: 0,
        }
    }
}
