//! SSH connection management
//!
//! Core module for establishing, authenticating, and managing SSH connections
//! using the russh library. Communicates with the UI layer via event channels.

pub mod auth;
pub mod client;
pub mod known_hosts;

use std::path::PathBuf;

use tokio::sync::mpsc;

/// Unique identifier for an SSH session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SessionId(pub u64);

/// Connection state machine.
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Authenticating,
    Connected,
    Error(String),
}

/// Authentication method configuration.
#[derive(Debug, Clone)]
pub enum AuthMethod {
    /// Password-based authentication.
    Password { password: String },
    /// Private key authentication, optionally with passphrase.
    Key {
        key_path: PathBuf,
        passphrase: Option<String>,
    },
    /// Keyboard-interactive authentication (e.g., OTP, two-factor).
    KeyboardInteractive,
}

/// Connection parameters for establishing an SSH session.
#[derive(Debug, Clone)]
pub struct SshConnectConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub auth_method: AuthMethod,
}

/// Events emitted from the SSH backend to the UI layer.
#[derive(Debug, Clone)]
pub enum SshEvent {
    /// Connection established successfully.
    Connected {
        session_id: SessionId,
    },
    /// Connection terminated.
    Disconnected {
        session_id: SessionId,
        reason: String,
    },
    /// Authentication banner from the server.
    AuthBanner {
        message: String,
    },
    /// First-time host key verification required.
    KeyVerificationRequired {
        host: String,
        key_fingerprint: String,
        key_type: String,
    },
    /// Host key has changed since last connection.
    KeyVerificationChanged {
        host: String,
        old_fingerprint: String,
        new_fingerprint: String,
    },
    /// An error occurred during connection or session.
    Error {
        session_id: SessionId,
        message: String,
    },
    /// General connection status change.
    StatusChanged {
        session_id: SessionId,
        status: ConnectionStatus,
    },
}

/// A channel sender for pushing SSH events to the UI layer.
pub type SshEventSender = mpsc::Sender<SshEvent>;

/// A channel receiver for consuming SSH events from the UI layer.
pub type SshEventReceiver = mpsc::Receiver<SshEvent>;

/// Create a new SSH event channel with the given buffer size.
pub fn ssh_event_channel(buffer: usize) -> (SshEventSender, SshEventReceiver) {
    mpsc::channel(buffer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_id_creation() {
        let id1 = SessionId(1);
        let id2 = SessionId(2);
        assert_ne!(id1, id2);
        assert_eq!(id1, SessionId(1));
    }

    #[test]
    fn test_connection_status_transition() {
        let states = vec![
            ConnectionStatus::Disconnected,
            ConnectionStatus::Connecting,
            ConnectionStatus::Authenticating,
            ConnectionStatus::Connected,
            ConnectionStatus::Error("test error".to_string()),
        ];
        assert_eq!(states.len(), 5);
        assert_ne!(states[0], states[1]);
        assert_ne!(states[1], states[2]);
        assert_ne!(states[2], states[3]);
        assert_eq!(
            format!("{:?}", states[4]),
            "Error(\"test error\")"
        );
    }

    #[test]
    fn test_ssh_event_channel_creation() {
        let (tx, mut rx) = ssh_event_channel(16);
        let event = SshEvent::StatusChanged {
            session_id: SessionId(1),
            status: ConnectionStatus::Connecting,
        };
        tx.try_send(event.clone()).unwrap();
        let received = rx.try_recv().unwrap();
        assert_eq!(format!("{:?}", received), format!("{:?}", event));
    }

    #[test]
    fn test_connection_status_debug_format() {
        assert_eq!(
            format!("{:?}", ConnectionStatus::Disconnected),
            "Disconnected"
        );
        assert_eq!(
            format!("{:?}", ConnectionStatus::Connecting),
            "Connecting"
        );
        assert_eq!(
            format!("{:?}", ConnectionStatus::Connected),
            "Connected"
        );
    }
}
