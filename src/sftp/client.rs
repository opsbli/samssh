//! SFTP client wrapper using russh-sftp
//!
//! Provides `SftpClient` for remote file operations via SFTP protocol.
//! Wraps `russh_sftp::client::SftpSession` for high-level access.

use russh::Channel;
use russh::client::Msg;
use russh_sftp::client::SftpSession;
use russh_sftp::protocol::FileType as SftpFileType;

use crate::sftp::{SftpEntry, SftpError, SftpSessionId, normalize_path};

static NEXT_SFTP_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

/// High-level SFTP client wrapping a russh-sftp session.
pub struct SftpClient {
    id: SftpSessionId,
    session: SftpSession,
    current_dir: String,
}

impl SftpClient {
    /// Create an SFTP client from an existing SSH channel by requesting the "sftp" subsystem.
    pub async fn from_channel(
        channel: Channel<Msg>,
    ) -> Result<Self, SftpError> {
        channel.request_subsystem(true, "sftp")
            .await
            .map_err(|e| SftpError::SessionError(format!("Failed to open SFTP subsystem: {}", e)))?;

        let stream = channel.into_stream();
        let session = SftpSession::new(stream)
            .await
            .map_err(|e| SftpError::SessionError(format!("Failed to initialize SFTP: {}", e)))?;

        let id = SftpSessionId(NEXT_SFTP_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst));

        Ok(Self {
            id,
            session,
            current_dir: "/".to_string(),
        })
    }

    /// Create from an already-initialized SftpSession.
    pub fn new(session: SftpSession) -> Self {
        let id = SftpSessionId(NEXT_SFTP_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst));
        Self {
            id,
            session,
            current_dir: "/".to_string(),
        }
    }

    /// Get the session ID.
    pub fn id(&self) -> SftpSessionId { self.id }

    /// Get the current remote directory.
    pub fn current_dir(&self) -> &str { &self.current_dir }

    /// Set the current remote directory.
    pub fn set_current_dir(&mut self, path: &str) {
        self.current_dir = normalize_path(path);
    }

    /// List entries in a remote directory.
    pub async fn list_dir(&self, path: &str) -> Result<Vec<SftpEntry>, SftpError> {
        let path = normalize_path(path);
        let mut read_dir = self.session.read_dir(&path)
            .await
            .map_err(|e| map_sftp_error(e, &path))?;

        let mut entries = Vec::new();
        loop {
            match read_dir.next() {
                Some(entry) => {
                    let file_name = entry.file_name();
                    if file_name == "." || file_name == ".." { continue; }

                    let metadata = entry.metadata();
                    entries.push(SftpEntry {
                        file_name,
                        is_dir: matches!(entry.file_type(), SftpFileType::Dir),
                        size: metadata.size.unwrap_or(0),
                        modified_unix: metadata.mtime.map(|t| t as u64),
                        uid: metadata.uid,
                        user: metadata.user,
                        gid: metadata.gid,
                        group: metadata.group,
                        permissions: metadata.permissions,
                    });
                }
                None => break,
            }
        }

        Ok(entries)
    }

    /// Get metadata for a single path.
    pub async fn stat(&self, path: &str) -> Result<SftpEntry, SftpError> {
        let path = normalize_path(path);
        let metadata = self.session.metadata(&path)
            .await
            .map_err(|e| map_sftp_error(e, &path))?;

        let file_name = path.rsplit('/').next().unwrap_or(&path).to_string();

        Ok(SftpEntry {
            file_name,
            is_dir: metadata.is_dir(),
            size: metadata.size.unwrap_or(0),
            modified_unix: metadata.mtime.map(|t| t as u64),
            uid: metadata.uid,
            user: metadata.user,
            gid: metadata.gid,
            group: metadata.group,
            permissions: metadata.permissions,
        })
    }

    /// Resolve a relative path to an absolute path.
    pub async fn canonicalize(&self, path: &str) -> Result<String, SftpError> {
        self.session.canonicalize(path)
            .await
            .map_err(|e| SftpError::OperationFailed(format!("Failed to canonicalize '{}': {}", path, e)))
    }

    /// Check if a path exists.
    pub async fn exists(&self, path: &str) -> Result<bool, SftpError> {
        self.session.try_exists(path)
            .await
            .map_err(|e| SftpError::OperationFailed(format!("Failed to check '{}': {}", path, e)))
    }

    /// Create a remote directory.
    pub async fn create_dir(&self, path: &str) -> Result<(), SftpError> {
        let path = normalize_path(path);
        self.session.create_dir(&path)
            .await
            .map_err(|e| SftpError::OperationFailed(format!("Failed to create dir '{}': {}", path, e)))
    }

    /// Rename a remote file or directory.
    pub async fn rename(&self, old_path: &str, new_path: &str) -> Result<(), SftpError> {
        let old = normalize_path(old_path);
        let new = normalize_path(new_path);
        self.session.rename(&old, &new)
            .await
            .map_err(|e| SftpError::OperationFailed(
                format!("Failed to rename '{}' -> '{}': {}", old, new, e)))
    }

    /// Remove a remote file.
    pub async fn remove_file(&self, path: &str) -> Result<(), SftpError> {
        let path = normalize_path(path);
        self.session.remove_file(&path)
            .await
            .map_err(|e| SftpError::OperationFailed(format!("Failed to remove file '{}': {}", path, e)))
    }

    /// Remove a remote directory (must be empty).
    pub async fn remove_dir(&self, path: &str) -> Result<(), SftpError> {
        let path = normalize_path(path);
        self.session.remove_dir(&path)
            .await
            .map_err(|e| SftpError::OperationFailed(format!("Failed to remove dir '{}': {}", path, e)))
    }

    /// Read the entire contents of a remote file.
    pub async fn read_file(&self, path: &str) -> Result<Vec<u8>, SftpError> {
        let path = normalize_path(path);
        self.session.read(&path)
            .await
            .map_err(|e| SftpError::OperationFailed(format!("Failed to read '{}': {}", path, e)))
    }

    /// Write data to a remote file (creates/truncates).
    pub async fn write_file(&self, path: &str, data: &[u8]) -> Result<(), SftpError> {
        let path = normalize_path(path);
        self.session.write(&path, data)
            .await
            .map_err(|e| SftpError::OperationFailed(format!("Failed to write '{}': {}", path, e)))
    }

    /// Close the SFTP session.
    pub async fn close(&self) -> Result<(), SftpError> {
        self.session.close()
            .await
            .map_err(|e| SftpError::SessionError(format!("Failed to close SFTP: {}", e)))
    }
}

fn map_sftp_error<E: std::fmt::Display>(e: E, path: &str) -> SftpError {
    let msg = e.to_string();
    if msg.contains("No such file") || msg.contains("not found") {
        SftpError::PathNotFound(path.to_string())
    } else if msg.contains("Permission denied") {
        SftpError::PermissionDenied(path.to_string())
    } else {
        SftpError::OperationFailed(msg)
    }
}
