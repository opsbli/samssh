//! SSH connection profile definition
//!
//! Profile represents a saved SSH connection configuration.
//! Sensitive fields (passwords, passphrases) are stored encrypted
//! and handled via the crypto module.

use serde::{Deserialize, Serialize};

/// Authentication method stored in configuration.
/// Sensitive credentials are encrypted separately via DPAPI.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StoredAuthMethod {
    /// Password-based auth (password encrypted in config store).
    Password,
    /// Private key authentication.
    Key {
        /// Path to the private key file.
        key_path: String,
        /// Optional passphrase (encrypted if stored).
        #[serde(skip_serializing_if = "Option::is_none")]
        passphrase_encrypted: Option<String>,
    },
    /// Keyboard-interactive authentication.
    KeyboardInteractive,
}

/// SSH connection profile.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Profile {
    /// Unique identifier.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Hostname or IP address.
    pub host: String,
    /// SSH port (default 22).
    #[serde(default = "default_port")]
    pub port: u16,
    /// SSH username.
    pub username: String,
    /// Authentication method.
    pub auth_method: StoredAuthMethod,
    /// Optional group for organizing profiles.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    /// Sort order within group.
    #[serde(default)]
    pub sort_order: u32,
    /// Encrypted password (if using password auth).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password_encrypted: Option<String>,
}

fn default_port() -> u16 {
    22
}

impl Profile {
    /// Create a new profile with default port 22.
    pub fn new(id: &str, name: &str, host: &str, username: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            host: host.to_string(),
            port: 22,
            username: username.to_string(),
            auth_method: StoredAuthMethod::Password,
            group: None,
            sort_order: 0,
            password_encrypted: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_default_port() {
        let p = Profile::new("test-1", "Test Server", "192.168.1.1", "admin");
        assert_eq!(p.port, 22);
        assert_eq!(p.id, "test-1");
        assert_eq!(p.name, "Test Server");
    }

    #[test]
    fn test_profile_json_roundtrip_password() {
        let profile = Profile {
            id: "srv-1".into(),
            name: "Web Server".into(),
            host: "web.example.com".into(),
            port: 2222,
            username: "deploy".into(),
            auth_method: StoredAuthMethod::Password,
            group: Some("Production".into()),
            sort_order: 1,
            password_encrypted: Some("encrypted_password_base64".into()),
        };

        let json = serde_json::to_string(&profile).unwrap();
        let deserialized: Profile = serde_json::from_str(&json).unwrap();

        assert_eq!(profile, deserialized);
    }

    #[test]
    fn test_profile_json_roundtrip_key() {
        let profile = Profile {
            id: "srv-2".into(),
            name: "Dev Box".into(),
            host: "dev.local".into(),
            port: 22,
            username: "dev".into(),
            auth_method: StoredAuthMethod::Key {
                key_path: "~/.ssh/id_ed25519".into(),
                passphrase_encrypted: None,
            },
            group: None,
            sort_order: 0,
            password_encrypted: None,
        };

        let json = serde_json::to_string_pretty(&profile).unwrap();
        let deserialized: Profile = serde_json::from_str(&json).unwrap();

        assert_eq!(profile, deserialized);
    }

    #[test]
    fn test_profile_json_roundtrip_kbi() {
        let profile = Profile::new("srv-3", "OTP Gateway", "gateway.example.com", "user");
        let json = serde_json::to_string(&profile).unwrap();
        let deserialized: Profile = serde_json::from_str(&json).unwrap();
        assert_eq!(profile, deserialized);
    }

    #[test]
    fn test_profile_serialization_omits_none() {
        let profile = Profile::new("no-group", "Server", "host", "user");
        let json = serde_json::to_string(&profile).unwrap();
        // Should not contain optional field keys when None
        assert!(!json.contains("\"password_encrypted\""));
        assert!(!json.contains("\"group\""));
    }
}
