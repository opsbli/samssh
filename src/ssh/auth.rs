//! SSH authentication handling
//!
//! Provides high-level authentication functions that wrap russh's
//! authentication methods: password, public key, and keyboard-interactive.

use std::sync::Arc;

use russh::client;

use crate::ssh::client::{SshClient, SshError};

/// Result of an authentication attempt.
#[derive(Debug, Clone, PartialEq)]
pub enum AuthResult {
    /// Authentication succeeded.
    Success,
    /// Authentication failed with an error message.
    Failed(String),
    /// Keyboard-interactive requires user input.
    KeyboardInteractiveRequired {
        name: String,
        instruction: String,
        prompts: Vec<(String, bool)>, // (prompt, is_secret)
    },
}

/// Perform password authentication.
pub async fn perform_password_auth(
    handle: &mut client::Handle<SshClient>,
    username: &str,
    password: &str,
) -> Result<bool, SshError> {
    tracing::debug!("Attempting password auth for user {}", username);
    SshClient::authenticate_password(handle, username, password).await
}

/// Perform public key authentication.
pub async fn perform_key_auth(
    handle: &mut client::Handle<SshClient>,
    username: &str,
    key_path: &std::path::Path,
    passphrase: Option<&str>,
) -> Result<bool, SshError> {
    tracing::debug!(
        "Attempting key auth for user {} with key {:?}",
        username,
        key_path
    );

    let key_pair = if let Some(pass) = passphrase {
        crate::ssh::client::load_private_key_with_passphrase(key_path, pass)?
    } else {
        crate::ssh::client::load_private_key(key_path)?
    };

    SshClient::authenticate_key(handle, username, Arc::new(key_pair)).await
}

/// Start keyboard-interactive authentication.
pub async fn perform_kbi_start(
    handle: &mut client::Handle<SshClient>,
    username: &str,
) -> Result<russh::client::KeyboardInteractiveAuthResponse, SshError> {
    tracing::debug!("Starting keyboard-interactive auth for user {}", username);
    SshClient::authenticate_kbi_start(handle, username).await
}

/// Respond to a keyboard-interactive challenge.
pub async fn perform_kbi_respond(
    handle: &mut client::Handle<SshClient>,
    responses: Vec<String>,
) -> Result<russh::client::KeyboardInteractiveAuthResponse, SshError> {
    tracing::debug!("Responding to keyboard-interactive challenge");
    SshClient::authenticate_kbi_respond(handle, responses).await
}

/// Attempt authentication with automatic fallback through methods.
///
/// Tries: 1. Public key (if key_path provided), 2. Password (if provided), 3. Keyboard-interactive
pub async fn authenticate_with_fallback(
    handle: &mut client::Handle<SshClient>,
    username: &str,
    password: Option<&str>,
    key_path: Option<&std::path::Path>,
    key_passphrase: Option<&str>,
) -> AuthResult {
    if let Some(kp) = key_path {
        match perform_key_auth(handle, username, kp, key_passphrase).await {
            Ok(true) => {
                tracing::info!("Key auth succeeded for {}", username);
                return AuthResult::Success;
            }
            Ok(false) => {
                tracing::warn!("Key auth failed for {} (server rejected key)", username);
            }
            Err(e) => {
                tracing::error!("Key auth error for {}: {}", username, e);
            }
        }
    }

    if let Some(pwd) = password {
        match perform_password_auth(handle, username, pwd).await {
            Ok(true) => {
                tracing::info!("Password auth succeeded for {}", username);
                return AuthResult::Success;
            }
            Ok(false) => {
                tracing::warn!("Password auth failed for {}", username);
                return AuthResult::Failed("Password authentication failed".to_string());
            }
            Err(e) => {
                tracing::error!("Password auth error for {}: {}", username, e);
                return AuthResult::Failed(format!("Password auth error: {}", e));
            }
        }
    }

    AuthResult::KeyboardInteractiveRequired {
        name: String::new(),
        instruction: "Keyboard-interactive authentication required".to_string(),
        prompts: vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_auth_result_debug() {
        let success = AuthResult::Success;
        assert_eq!(format!("{:?}", success), "Success");

        let failed = AuthResult::Failed("bad password".to_string());
        assert_eq!(format!("{:?}", failed), "Failed(\"bad password\")");
    }

    #[test]
    fn test_auth_result_partial_eq() {
        assert_eq!(AuthResult::Success, AuthResult::Success);
        assert_ne!(
            AuthResult::Success,
            AuthResult::Failed("error".to_string())
        );
    }

    #[test]
    fn test_load_private_key_nonexistent() {
        let path = Path::new("/nonexistent/path/key.pem");
        let result = crate::ssh::client::load_private_key(path);
        assert!(result.is_err());
        match result {
            Err(SshError::AuthFailed(msg)) => {
                assert!(msg.contains("Failed to read key file"));
            }
            _ => panic!("Expected AuthFailed error"),
        }
    }

    #[test]
    fn test_load_private_key_invalid_format() {
        let dir = std::env::temp_dir();
        let path = dir.join("test_invalid_key.pem");
        std::fs::write(&path, b"not a valid key").unwrap();

        let result = crate::ssh::client::load_private_key(&path);
        assert!(result.is_err());

        std::fs::remove_file(&path).ok();
    }
}
