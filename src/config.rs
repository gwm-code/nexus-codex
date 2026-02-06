use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::provider::{ProviderConfig, ProviderKind, ProviderSettings};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub provider: ProviderKind,
    pub api_key: Option<String>,
    pub dry_run: bool,
    pub gemini: ProviderConfig,
    pub openrouter: ProviderConfig,
    pub opencode: ProviderConfig,
    pub claude: ProviderConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            provider: ProviderKind::Gemini,
            api_key: None,
            dry_run: true,
            gemini: ProviderConfig::default(),
            openrouter: ProviderConfig::default(),
            opencode: ProviderConfig::default(),
            claude: ProviderConfig::default(),
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let path = config_path().unwrap_or_else(|| PathBuf::from("nexus.toml"));
        if let Ok(contents) = std::fs::read_to_string(&path) {
            toml::from_str(&contents).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn path() -> Option<PathBuf> {
        config_path()
    }

    pub fn save(&self, path: &Path) -> Result<(), std::io::Error> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let data = toml::to_string_pretty(self).unwrap_or_default();
        std::fs::write(path, data)
    }

    pub fn provider_settings(&self) -> ProviderSettings {
        let fallback_key = self.api_key.clone();
        let provider_config = match self.provider {
            ProviderKind::Gemini => &self.gemini,
            ProviderKind::OpenRouter => &self.openrouter,
            ProviderKind::OpenCode => &self.opencode,
            ProviderKind::Claude => &self.claude,
        };

        ProviderSettings {
            api_key: provider_config.api_key.clone().or(fallback_key),
            model: provider_config.model.clone(),
            base_url: provider_config.base_url.clone(),
        }
    }
}

fn config_path() -> Option<PathBuf> {
    let explicit = std::env::var("NEXUS_CONFIG").ok();
    if let Some(path) = explicit {
        return Some(PathBuf::from(path));
    }

    dirs::config_dir().map(|dir| dir.join("nexus").join("nexus.toml"))
}
