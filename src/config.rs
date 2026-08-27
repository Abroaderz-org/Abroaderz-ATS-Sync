use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub input_directory: String,
    pub excel_output_path: String,
    pub csv_output_path: String,
    pub batch_limit: usize,
    pub license_key: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            input_directory: "./resumes".to_string(),
            excel_output_path: "./Abroaderz_Candidates.xlsx".to_string(),
            csv_output_path: "./Abroaderz_Candidates.csv".to_string(),
            batch_limit: 100,
            license_key: "abroaderz_admin".to_string(),
        }
    }
}

impl AppConfig {
    pub fn load() -> Self {
        Self::load_or_create("config.toml")
    }

    pub fn load_or_create<P: AsRef<Path>>(config_path: P) -> Self {
        let path = config_path.as_ref();

        if path.exists() {
            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(config) = toml::from_str::<AppConfig>(&content) {
                    return config;
                }
            }
        }

        // Generate default config file if missing or invalid
        let default_config = AppConfig::default();
        if let Ok(toml_str) = toml::to_string_pretty(&default_config) {
            let _ = fs::write(path, toml_str);
        }

        default_config
    }
}