use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub defaults: DefaultConfig,
    #[serde(default)]
    pub profiles: HashMap<String, ProfileConfig>,
    #[serde(default)]
    pub aliases: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultConfig {
    #[serde(default = "default_region")]
    pub region: String,
    #[serde(default = "default_output")]
    pub output: String,
    #[serde(default = "default_max_events")]
    pub max_events: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileConfig {
    pub assume_role: Option<String>,
    pub region: Option<String>,
}

fn default_region() -> String {
    "us-east-1".to_string()
}

fn default_output() -> String {
    "colored".to_string()
}

fn default_max_events() -> usize {
    1000
}

impl Default for Config {
    fn default() -> Self {
        Self {
            defaults: DefaultConfig {
                region: default_region(),
                output: default_output(),
                max_events: default_max_events(),
            },
            profiles: HashMap::new(),
            aliases: HashMap::new(),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if config_path.exists() {
            let contents = std::fs::read_to_string(&config_path)?;
            let config: Config = toml::from_str(&contents)?;
            Ok(config)
        } else {
            Ok(Config::default())
        }
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;

        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let contents = toml::to_string_pretty(&self)?;
        std::fs::write(&config_path, contents)?;

        Ok(())
    }

    fn config_path() -> Result<PathBuf> {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))?;
        let mut path = PathBuf::from(home);
        path.push(".config");
        path.push("cwl");
        path.push("config.toml");
        Ok(path)
    }
}