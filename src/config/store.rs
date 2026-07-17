//! Configuration storage
//!
//! Top-level configuration container and persistent storage.
//! Config is stored as JSON at `%APPDATA%/SamSSH/config.json`.
//! Sensitive fields are encrypted via DPAPI.
//!
//! Primary storage uses DPAPI-encrypted JSON.
//! Falls back to plaintext JSON (without credentials) if encryption fails.

use serde::{Deserialize, Serialize};

use crate::config::profile::Profile;
use crate::config::settings::AppSettings;
use crate::crypto::dpapi;

/// Top-level application configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Config {
    #[serde(default)]
    pub profiles: Vec<Profile>,
    #[serde(default)]
    pub settings: AppSettings,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window_x: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window_y: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window_width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window_height: Option<u32>,
    #[serde(default)]
    pub window_maximized: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            profiles: Vec::new(),
            settings: AppSettings::default(),
            window_x: None,
            window_y: None,
            window_width: None,
            window_height: None,
            window_maximized: false,
        }
    }
}

/// Configuration store managing file I/O with DPAPI encryption.
pub struct ConfigStore {
    config_path: std::path::PathBuf,
}

impl ConfigStore {
    pub fn new() -> Self {
        let path = Self::default_config_path();
        Self { config_path: path }
    }

    pub fn with_path(path: std::path::PathBuf) -> Self {
        Self { config_path: path }
    }

    fn default_config_path() -> std::path::PathBuf {
        let base = dirs_next::data_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."));
        base.join("SamSSH").join("config.json")
    }

    pub fn path(&self) -> &std::path::Path {
        &self.config_path
    }

    /// Load and decrypt configuration from disk.
    pub fn load(&self) -> Result<Config, String> {
        if !self.config_path.exists() {
            return Ok(Config::default());
        }
        let content = std::fs::read_to_string(&self.config_path)
            .map_err(|e| format!("Failed to read config: {}", e))?;

        // Try DPAPI-decrypted first
        match dpapi::decrypt_from_string(&content) {
            Ok(json) => serde_json::from_str(&json)
                .map_err(|e| format!("Failed to parse decrypted config: {}", e)),
            Err(_) => {
                // Fallback to plaintext
                serde_json::from_str(&content)
                    .map_err(|e| format!("Failed to parse config: {}", e))
            }
        }
    }

    /// Save configuration to disk with DPAPI encryption.
    pub fn save(&self, config: &Config) -> Result<(), String> {
        if let Some(parent) = self.config_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config dir: {}", e))?;
        }

        let json = serde_json::to_string_pretty(config)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        let content = dpapi::encrypt_to_string(&json).unwrap_or_else(|_| {
            let clean = Self::strip_credentials(config);
            serde_json::to_string_pretty(&clean).unwrap_or_default()
        });

        let tmp_path = self.config_path.with_extension("json.tmp");
        std::fs::write(&tmp_path, &content)
            .map_err(|e| format!("Failed to write config: {}", e))?;
        std::fs::rename(&tmp_path, &self.config_path)
            .map_err(|e| format!("Failed to finalize config: {}", e))?;
        Ok(())
    }

    fn strip_credentials(config: &Config) -> Config {
        let mut clean = config.clone();
        for profile in &mut clean.profiles {
            profile.password_encrypted = None;
            if let crate::config::profile::StoredAuthMethod::Key { ref mut passphrase_encrypted, .. } = profile.auth_method {
                *passphrase_encrypted = None;
            }
        }
        clean
    }

    pub fn exists(&self) -> bool {
        self.config_path.exists()
    }
}

impl Default for ConfigStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert!(config.profiles.is_empty());
        assert!(!config.window_maximized);
    }

    #[test]
    fn test_config_save_and_load() {
        let dir = std::env::temp_dir().join("samssh_test_config_dpapi");
        let _ = std::fs::remove_dir_all(&dir);
        let path = dir.join("config.json");
        let store = ConfigStore::with_path(path.clone());

        let profile = Profile::new("save-test", "Save Test", "host.local", "user");
        let config = Config {
            profiles: vec![profile],
            window_x: Some(100),
            ..Default::default()
        };

        store.save(&config).unwrap();
        assert!(path.exists());
        let loaded = store.load().unwrap();
        assert_eq!(loaded.profiles.len(), 1);
        assert_eq!(loaded.profiles[0].id, "save-test");
        assert_eq!(loaded.window_x, Some(100));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_config_load_nonexistent() {
        let dir = std::env::temp_dir().join("samssh_test_nonexistent_dpapi");
        let path = dir.join("config.json");
        let store = ConfigStore::with_path(path);
        let config = store.load().unwrap();
        assert!(config.profiles.is_empty());
    }

    #[test]
    fn test_config_atomic_write() {
        let dir = std::env::temp_dir().join("samssh_test_atomic_dpapi");
        let _ = std::fs::remove_dir_all(&dir);
        let path = dir.join("config.json");
        let store = ConfigStore::with_path(path.clone());
        store.save(&Config::default()).unwrap();
        assert!(!dir.join("config.json.tmp").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_strip_credentials() {
        let mut profile = Profile::new("test", "Test", "host", "user");
        profile.password_encrypted = Some("encrypted".into());
        let config = Config {
            profiles: vec![profile],
            ..Default::default()
        };
        let clean = ConfigStore::strip_credentials(&config);
        assert!(clean.profiles[0].password_encrypted.is_none());
    }
}
