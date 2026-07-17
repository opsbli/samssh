//! SSH client wrapper
//!
//! Provides the `SshClient` struct that wraps russh's client functionality,
//! implementing the `russh::client::Handler` trait for event callbacks.
//! Uses `tokio::sync::mpsc` to push events to the UI layer.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use async_trait::async_trait;
use russh::client::{self, Handler, Msg, Session};
use russh::Channel;
use russh::keys::key::{KeyPair, PublicKey};

use crate::ssh::{ConnectionStatus, SshConnectConfig, SshEvent, SshEventSender, SessionId};

static NEXT_SESSION_ID: AtomicU64 = AtomicU64::new(1);

/// Error type for SSH operations.
#[derive(Debug, thiserror::Error)]
pub enum SshError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Authentication failed: {0}")]
    AuthFailed(String),
    #[error("Channel error: {0}")]
    ChannelError(String),
    #[error("Disconnected: {0}")]
    Disconnected(String),
}

impl From<russh::Error> for SshError {
    fn from(e: russh::Error) -> Self {
        SshError::ConnectionFailed(e.to_string())
    }
}

/// Manages an SSH connection lifecycle.
///
/// Wraps the russh client handle and implements the Handler trait
/// to receive callbacks (host key verification, disconnect, etc.).
pub struct SshClient {
    session_id: SessionId,
    config: SshConnectConfig,
    event_tx: SshEventSender,
    /// Whether host key verification passed.
    host_key_verified: bool,
}

impl SshClient {
    /// Create a new SSH client instance with the given config.
    pub fn new(config: SshConnectConfig) -> Self {
        let session_id = SessionId(NEXT_SESSION_ID.fetch_add(1, Ordering::SeqCst));
        let (event_tx, _) = crate::ssh::ssh_event_channel(64);
        Self {
            session_id,
            config,
            event_tx,
            host_key_verified: false,
        }
    }

    /// Create with explicit event sender, for sharing with the UI layer.
    pub fn with_channel(config: SshConnectConfig, event_tx: SshEventSender) -> Self {
        let session_id = SessionId(NEXT_SESSION_ID.fetch_add(1, Ordering::SeqCst));
        Self {
            session_id,
            config,
            event_tx,
            host_key_verified: false,
        }
    }

    /// Session ID for this client.
    pub fn session_id(&self) -> SessionId {
        self.session_id
    }

    /// Event sender for pushing events to the UI.
    pub fn event_sender(&self) -> SshEventSender {
        self.event_tx.clone()
    }

    /// Get the connect config.
    pub fn connect_config(&self) -> &SshConnectConfig {
        &self.config
    }

    /// Whether host key has been verified.
    pub fn is_host_key_verified(&self) -> bool {
        self.host_key_verified
    }

    /// Send a status event to the UI layer.
    fn send_event(&self, event: SshEvent) {
        let _ = self.event_tx.try_send(event);
    }

    /// Build the russh client config.
    fn build_client_config() -> Arc<client::Config> {
        Arc::new(client::Config::default())
    }

    /// Connect to the SSH server.
    ///
    /// This performs TCP connection + SSH key exchange + host key verification.
    /// Consumes the client (moves it into the async runtime).
    pub async fn connect(self) -> Result<client::Handle<SshClient>, SshError> {
        self.send_event(SshEvent::StatusChanged {
            session_id: self.session_id,
            status: ConnectionStatus::Connecting,
        });

        let cfg = Self::build_client_config();
        let addr = format!("{}:{}", self.config.host, self.config.port);

        let handle = client::connect(cfg, addr.as_str(), self).await
            .map_err(|e| SshError::ConnectionFailed(e.to_string()))?;

        Ok(handle)
    }

    /// Perform password authentication.
    ///
    /// Returns true if authentication succeeded.
    pub async fn authenticate_password(
        handle: &mut client::Handle<SshClient>,
        username: &str,
        password: &str,
    ) -> Result<bool, SshError> {
        handle.authenticate_password(username, password)
            .await
            .map_err(|e| SshError::AuthFailed(e.to_string()))
    }

    /// Perform public key authentication.
    ///
    /// `key` should be loaded via `russh::keys::load_secret_key()`.
    pub async fn authenticate_key(
        handle: &mut client::Handle<SshClient>,
        username: &str,
        key: Arc<KeyPair>,
    ) -> Result<bool, SshError> {
        handle.authenticate_publickey(username, key)
            .await
            .map_err(|e| SshError::AuthFailed(e.to_string()))
    }

    /// Start keyboard-interactive authentication.
    pub async fn authenticate_kbi_start(
        handle: &mut client::Handle<SshClient>,
        username: &str,
    ) -> Result<russh::client::KeyboardInteractiveAuthResponse, SshError> {
        handle.authenticate_keyboard_interactive_start(username, None::<String>)
            .await
            .map_err(|e| SshError::AuthFailed(e.to_string()))
    }

    /// Respond to a keyboard-interactive challenge.
    pub async fn authenticate_kbi_respond(
        handle: &mut client::Handle<SshClient>,
        responses: Vec<String>,
    ) -> Result<russh::client::KeyboardInteractiveAuthResponse, SshError> {
        handle.authenticate_keyboard_interactive_respond(responses)
            .await
            .map_err(|e| SshError::AuthFailed(e.to_string()))
    }

    /// Open a session channel (for shell, exec, or SFTP).
    pub async fn open_session(
        handle: &client::Handle<SshClient>,
    ) -> Result<Channel<Msg>, SshError> {
        handle.channel_open_session()
            .await
            .map_err(|e| SshError::ChannelError(e.to_string()))
    }

    /// Disconnect from the server.
    pub async fn disconnect(
        handle: &client::Handle<SshClient>,
    ) -> Result<(), SshError> {
        handle.disconnect(russh::Disconnect::ByApplication, "Client disconnect", "en")
            .await
            .map_err(|e| SshError::Disconnected(e.to_string()))
    }
}

#[async_trait]
impl Handler for SshClient {
    type Error = russh::Error;

    /// Called when the server's host key needs to be verified.
    async fn check_server_key(
        &mut self,
        server_public_key: &PublicKey,
    ) -> Result<bool, Self::Error> {
        let fingerprint = server_public_key.fingerprint();
        let key_type = format!("{:?}", server_public_key);

        let host = &self.config.host;
        match crate::ssh::known_hosts::verify_host_key(host, server_public_key) {
            Ok(true) => {
                self.host_key_verified = true;
                self.send_event(SshEvent::StatusChanged {
                    session_id: self.session_id,
                    status: ConnectionStatus::Authenticating,
                });
                Ok(true)
            }
            Ok(false) => {
                self.send_event(SshEvent::KeyVerificationChanged {
                    host: host.clone(),
                    old_fingerprint: String::new(),
                    new_fingerprint: fingerprint,
                });
                Ok(false)
            }
            Err(_) => {
                self.send_event(SshEvent::KeyVerificationRequired {
                    host: host.clone(),
                    key_fingerprint: fingerprint,
                    key_type,
                });
                Ok(false)
            }
        }
    }

    /// Called when the server sends an authentication banner.
    async fn auth_banner(
        &mut self,
        banner: &str,
        _session: &mut Session,
    ) -> Result<(), Self::Error> {
        self.send_event(SshEvent::AuthBanner {
            message: banner.to_string(),
        });
        Ok(())
    }

    /// Called when the connection has been disconnected.
    async fn disconnected(
        &mut self,
        _reason: russh::client::DisconnectReason<Self::Error>,
    ) -> Result<(), Self::Error> {
        self.send_event(SshEvent::Disconnected {
            session_id: self.session_id,
            reason: "Connection closed".to_string(),
        });
        Ok(())
    }
}

/// Load a private key file for authentication.
pub fn load_private_key(path: &std::path::Path) -> Result<KeyPair, SshError> {
    use std::fs;
    let data = fs::read_to_string(path)
        .map_err(|e| SshError::AuthFailed(format!("Failed to read key file: {}", e)))?;
    russh::keys::load_secret_key(&data, None)
        .map_err(|e| SshError::AuthFailed(format!("Failed to parse key: {}", e)))
}

/// Load a private key file with passphrase.
pub fn load_private_key_with_passphrase(
    path: &std::path::Path,
    passphrase: &str,
) -> Result<KeyPair, SshError> {
    use std::fs;
    let data = fs::read_to_string(path)
        .map_err(|e| SshError::AuthFailed(format!("Failed to read key file: {}", e)))?;
    russh::keys::load_secret_key(&data, Some(passphrase))
        .map_err(|e| SshError::AuthFailed(format!("Failed to parse key: {}", e)))
}
