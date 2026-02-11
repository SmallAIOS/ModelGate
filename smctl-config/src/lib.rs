use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Three-tier configuration: CLI flags > workspace config > user config.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SmctlConfig {
    /// User-level config (~/.config/smctl/config.toml)
    #[serde(default)]
    pub user: UserConfig,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserConfig {
    /// Default editor for `smctl config edit`
    pub editor: Option<String>,
    /// Default log level
    pub log_level: Option<String>,
    /// Disable color output
    #[serde(default)]
    pub no_color: bool,
}

impl SmctlConfig {
    /// Load user config from the default location (~/.config/smctl/config.toml).
    pub fn load_user_config() -> Result<Self> {
        let path = Self::user_config_path()?;
        if path.exists() {
            let content = std::fs::read_to_string(&path).context("failed to read user config")?;
            let user: UserConfig =
                toml::from_str(&content).context("failed to parse user config")?;
            Ok(Self { user })
        } else {
            Ok(Self::default())
        }
    }

    /// Get the user config directory path.
    pub fn user_config_dir() -> Result<PathBuf> {
        let dir = dirs_path().join("smctl");
        Ok(dir)
    }

    /// Get the user config file path.
    pub fn user_config_path() -> Result<PathBuf> {
        Ok(Self::user_config_dir()?.join("config.toml"))
    }

    /// Get a config value by dotted key path.
    pub fn get(&self, key: &str) -> Option<String> {
        match key {
            "user.editor" => self.user.editor.clone(),
            "user.log_level" => self.user.log_level.clone(),
            "user.no_color" => Some(self.user.no_color.to_string()),
            _ => None,
        }
    }

    /// Set a config value by dotted key path.
    pub fn set(&mut self, key: &str, value: &str) -> Result<()> {
        match key {
            "user.editor" => self.user.editor = Some(value.to_string()),
            "user.log_level" => self.user.log_level = Some(value.to_string()),
            "user.no_color" => {
                self.user.no_color = value.parse().context("expected true or false")?
            }
            _ => anyhow::bail!("unknown config key: {key}"),
        }
        Ok(())
    }

    /// Save user config to disk.
    pub fn save_user_config(&self) -> Result<()> {
        let path = Self::user_config_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).context("failed to create config directory")?;
        }
        let content = toml::to_string_pretty(&self.user).context("failed to serialize config")?;
        std::fs::write(&path, content).context("failed to write config")?;
        Ok(())
    }

    /// Show effective configuration as a formatted string.
    pub fn show(&self) -> String {
        toml::to_string_pretty(&self.user).unwrap_or_default()
    }
}

/// Resolve XDG-style config directory.
fn dirs_path() -> PathBuf {
    if let Ok(config) = std::env::var("XDG_CONFIG_HOME") {
        PathBuf::from(config)
    } else if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".config")
    } else {
        PathBuf::from(".config")
    }
}

/// Find the workspace root by walking up from `start` looking for `.smctl/workspace.toml`.
pub fn find_workspace_root(start: &Path) -> Option<PathBuf> {
    let mut current = start.to_path_buf();
    loop {
        if current.join(".smctl").join("workspace.toml").exists() {
            return Some(current);
        }
        if !current.pop() {
            return None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = SmctlConfig::default();
        assert!(config.user.editor.is_none());
        assert!(!config.user.no_color);
    }

    #[test]
    fn test_get_set_config() {
        let mut config = SmctlConfig::default();
        config.set("user.editor", "vim").unwrap();
        assert_eq!(config.get("user.editor"), Some("vim".to_string()));
    }

    #[test]
    fn test_set_unknown_key() {
        let mut config = SmctlConfig::default();
        assert!(config.set("unknown.key", "value").is_err());
    }

    #[test]
    fn test_find_workspace_root_none() {
        let dir = tempfile::tempdir().unwrap();
        assert!(find_workspace_root(dir.path()).is_none());
    }
}
