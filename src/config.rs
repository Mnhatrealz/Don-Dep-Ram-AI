use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    pub api_key: String,
    pub base_url: String,
    pub model: String,
    pub temperature: f32,
    pub max_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationConfig {
    pub auto_clean_enabled: bool,
    pub ram_threshold_percent: u64,
    pub check_interval_seconds: u64,
    pub timer_resolution_boost: bool,
    pub clean_standby: bool,
    pub clean_workingset: bool,
    pub clean_temp_files: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub ai: AiConfig,
    pub automation: AutomationConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            ai: AiConfig {
                api_key: "sk-xt-6326e90bccc399cd6cf674b37ac9f3eef99ff46740e067bf".to_string(),
                base_url: "https://api.xkiro.com/v1".to_string(),
                model: "qwen/qwen3.5-omni-flash:free".to_string(),
                temperature: 0.3,
                max_tokens: 1024,
            },
            automation: AutomationConfig {
                auto_clean_enabled: true,
                ram_threshold_percent: 85,
                check_interval_seconds: 15,
                timer_resolution_boost: true,
                clean_standby: true,
                clean_workingset: true,
                clean_temp_files: true,
            },
        }
    }
}

impl AppConfig {
    pub fn load_or_default() -> Self {
        let path = Path::new("config.json");
        if path.exists() {
            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(cfg) = serde_json::from_str::<AppConfig>(&content) {
                    return cfg;
                }
            }
        }
        let default_cfg = AppConfig::default();
        let _ = fs::write(path, serde_json::to_string_pretty(&default_cfg).unwrap_or_default());
        default_cfg
    }
}
