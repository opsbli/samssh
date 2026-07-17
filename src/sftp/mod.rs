//! SFTP file transfer
//!
//! SFTP client and file transfer operations for remote file management.
//! Wraps russh-sftp for high-level file operations.

pub mod client;
pub mod transfer;

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};

static NEXT_SESSION_ID: AtomicU64 = AtomicU64::new(1);

/// Unique identifier for an SFTP session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SftpSessionId(pub u64);

/// A single file/directory entry from a remote directory listing.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SftpEntry {
    /// File name (not full path).
    pub file_name: String,
    /// Whether this entry is a directory.
    pub is_dir: bool,
    /// File size in bytes.
    pub size: u64,
    /// Last modification time (Unix timestamp).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_unix: Option<u64>,
    /// Owner user ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uid: Option<u32>,
    /// Owner user name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    /// Group ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gid: Option<u32>,
    /// Group name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    /// Unix permissions (e.g., 0o755).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<u32>,
}

impl SftpEntry {
    /// Create a new directory entry.
    pub fn dir(name: impl Into<String>) -> Self {
        Self {
            file_name: name.into(),
            is_dir: true,
            size: 0,
            modified_unix: None,
            uid: None,
            user: None,
            gid: None,
            group: None,
            permissions: None,
        }
    }

    /// Create a new file entry.
    pub fn file(name: impl Into<String>, size: u64) -> Self {
        Self {
            file_name: name.into(),
            is_dir: false,
            size,
            modified_unix: None,
            uid: None,
            user: None,
            gid: None,
            group: None,
            permissions: None,
        }
    }

    /// Full path by joining with a base directory.
    pub fn full_path(&self, base_dir: &str) -> String {
        if base_dir.ends_with('/') {
            format!("{}{}", base_dir, self.file_name)
        } else {
            format!("{}/{}", base_dir, self.file_name)
        }
    }
}

/// Error type for SFTP operations.
#[derive(Debug, thiserror::Error)]
pub enum SftpError {
    #[error("SFTP session error: {0}")]
    SessionError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Not connected")]
    NotConnected,
    #[error("Path not found: {0}")]
    PathNotFound(String),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("Operation failed: {0}")]
    OperationFailed(String),
}

/// Current status of a file transfer task.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransferStatus {
    Queued,
    Transferring,
    Paused,
    Completed,
    Failed(String),
    Cancelled,
}

/// A file transfer task (upload or download).
#[derive(Debug, Clone)]
pub struct TransferTask {
    pub id: u64,
    pub local_path: PathBuf,
    pub remote_path: String,
    pub direction: TransferDirection,
    pub total_bytes: u64,
    pub transferred_bytes: u64,
    pub status: TransferStatus,
}

/// Direction of a transfer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransferDirection {
    Upload,
    Download,
}

/// Sort entries with directories first, then alphabetically by name.
pub fn sort_entries(entries: &mut [SftpEntry]) {
    entries.sort_by(|a, b| {
        match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.file_name.to_lowercase().cmp(&b.file_name.to_lowercase()),
        }
    });
}

/// Normalize a remote path (remove trailing slash except for root).
pub fn normalize_path(path: &str) -> String {
    let path = path.trim();
    if path == "/" {
        return "/".to_string();
    }
    let p = path.trim_end_matches('/');
    if p.is_empty() {
        "/".to_string()
    } else {
        p.to_string()
    }
}

/// Join two path components with a '/'.
pub fn join_path(base: &str, name: &str) -> String {
    let base = normalize_path(base);
    if base == "/" {
        format!("/{}", name.trim_start_matches('/'))
    } else {
        format!("{}/{}", base, name.trim_start_matches('/'))
    }
}

/// Generate a new transfer task ID.
pub fn next_task_id() -> u64 {
    NEXT_SESSION_ID.fetch_add(1, Ordering::SeqCst)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_path_root() {
        assert_eq!(normalize_path("/"), "/");
        assert_eq!(normalize_path("//"), "/");
    }

    #[test]
    fn test_normalize_path_trailing_slash() {
        assert_eq!(normalize_path("/home/user/"), "/home/user");
        assert_eq!(normalize_path("/home/user///"), "/home/user");
    }

    #[test]
    fn test_normalize_path_no_slash() {
        assert_eq!(normalize_path("/home/user"), "/home/user");
    }

    #[test]
    fn test_join_path_base_with_slash() {
        assert_eq!(join_path("/home/", "file.txt"), "/home/file.txt");
    }

    #[test]
    fn test_join_path_root() {
        assert_eq!(join_path("/", "file.txt"), "/file.txt");
    }

    #[test]
    fn test_join_path_no_trailing_slash() {
        assert_eq!(join_path("/home/user", "doc.txt"), "/home/user/doc.txt");
    }

    #[test]
    fn test_sort_entries_dirs_first() {
        let mut entries = vec![
            SftpEntry::file("b.txt", 100),
            SftpEntry::dir("a_dir"),
            SftpEntry::file("a.txt", 50),
            SftpEntry::dir("z_dir"),
        ];
        sort_entries(&mut entries);
        assert!(entries[0].is_dir);
        assert!(entries[1].is_dir);
        assert!(!entries[2].is_dir);
        assert!(!entries[3].is_dir);
        assert_eq!(entries[2].file_name, "a.txt");
        assert_eq!(entries[3].file_name, "b.txt");
    }

    #[test]
    fn test_sftp_entry_full_path() {
        let entry = SftpEntry::file("test.txt", 100);
        assert_eq!(entry.full_path("/home/user"), "/home/user/test.txt");
        assert_eq!(entry.full_path("/home/user/"), "/home/user/test.txt");
    }

    #[test]
    fn test_sftp_entry_serde_roundtrip() {
        let entry = SftpEntry {
            file_name: "file.bin".into(),
            is_dir: false,
            size: 1024,
            modified_unix: Some(1700000000),
            uid: Some(1000),
            user: Some("alice".into()),
            gid: Some(100),
            group: Some("staff".into()),
            permissions: Some(0o644),
        };
        let json = serde_json::to_string(&entry).unwrap();
        let deserialized: SftpEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(entry, deserialized);
    }
}
