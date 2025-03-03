use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::os::unix::fs::FileExt;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub data_dir: String,
    pub btc_api_url: String,
    pub eth_api_url: String,
    pub enable_testnet: bool,
    pub enable_2fa: bool,
    pub auto_lock_minutes: u32,
    pub supported_coins: Vec<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let data_dir = home_dir
            .join(".safecoin-wallet")
            .to_string_lossy()
            .into_owned();

        Self {
            data_dir,
            btc_api_url: "".to_string(),
            eth_api_url: "".to_string(),
            enable_testnet: true, // Use testnet for development
            enable_2fa: false,
            auto_lock_minutes: 15,
            supported_coins: vec!["bitcoin".to_string(), "ethereum".to_string()],
        }
    }
}

impl AppConfig {
    pub fn load() -> Result<Self, Box<dyn Error>> {
        let config_path = Self::get_config_path();

        if !config_path.exists() {
            let config = Self::default();
            config.save()?;
            return Ok(config);
        }

        let mut file = File::open(config_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let config: AppConfig = serde_json::from_str(&contents)?;
        Ok(config)
    }

    pub fn save(&self) -> Result<(), Box<dyn Error>> {
        let config_path = Self::get_config_path();

        // Ensure data directory exists
        fs::create_dir_all(Path::new(&self.data_dir))?;

        // Ensure parent directory for config exists
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let config_json = serde_json::to_string_pretty(self)?;
        let mut file = File::create(config_path)?;
        file.write_all(config_json.as_bytes())?;

        Ok(())
    }

    fn get_config_path() -> PathBuf {
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home_dir.join(".safecoin-wallet").join("config.json")
    }
}
