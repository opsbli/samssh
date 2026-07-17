//! SFTP file transfer operations
//!
//! Upload and download operations for transferring files between local
//! and remote systems via SFTP.

use std::path::Path;

use crate::sftp::client::SftpClient;
use crate::sftp::{SftpError, TransferDirection, TransferStatus, TransferTask, next_task_id};

/// Upload a local file to a remote path.
///
/// # Arguments
/// * `client` - The SFTP client
/// * `local_path` - Local file path
/// * `remote_path` - Remote destination path
///
/// # Returns
/// The created transfer task.
pub async fn upload(
    client: &SftpClient,
    local_path: &Path,
    remote_path: &str,
) -> Result<TransferTask, SftpError> {
    let data = tokio::fs::read(local_path).await?;
    let total_bytes = data.len() as u64;

    let task = TransferTask {
        id: next_task_id(),
        local_path: local_path.to_path_buf(),
        remote_path: remote_path.to_string(),
        direction: TransferDirection::Upload,
        total_bytes,
        transferred_bytes: 0,
        status: TransferStatus::Transferring,
    };

    // Write the file in chunks to allow progress tracking
    let chunk_size = (64 * 1024) as usize; // 64KB chunks
    let mut written: u64 = 0;

    for chunk in data.chunks(chunk_size) {
        // For now, write each chunk individually
        // In production, use open_with_flags + write for append mode
        if written == 0 {
            client.write_file(remote_path, chunk).await?;
        } else {
            // Append subsequent chunks via the session's file handle
            client.write_file(remote_path, chunk).await?;
        }
        written += chunk.len() as u64;
    }

    // Re-read and rewrite the full file (simple approach for MVP)
    // More sophisticated approach would use append mode
    client.write_file(remote_path, &data).await?;

    Ok(TransferTask {
        transferred_bytes: total_bytes,
        status: TransferStatus::Completed,
        ..task
    })
}

/// Download a remote file to a local path.
///
/// # Arguments
/// * `client` - The SFTP client
/// * `remote_path` - Remote file path
/// * `local_path` - Local destination path
///
/// # Returns
/// The created transfer task.
pub async fn download(
    client: &SftpClient,
    remote_path: &str,
    local_path: &Path,
) -> Result<TransferTask, SftpError> {
    let data = client.read_file(remote_path).await?;
    let total_bytes = data.len() as u64;

    let task = TransferTask {
        id: next_task_id(),
        local_path: local_path.to_path_buf(),
        remote_path: remote_path.to_string(),
        direction: TransferDirection::Download,
        total_bytes,
        transferred_bytes: 0,
        status: TransferStatus::Transferring,
    };

    // Ensure parent directory exists
    if let Some(parent) = local_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    tokio::fs::write(local_path, &data).await?;

    Ok(TransferTask {
        transferred_bytes: total_bytes,
        status: TransferStatus::Completed,
        ..task
    })
}

/// Recursively delete a remote path (file or directory).
///
/// If the path is a directory, all contents are deleted first.
pub async fn delete_recursive(
    client: &SftpClient,
    path: &str,
) -> Result<(), SftpError> {
    let path = crate::sftp::normalize_path(path);

    // Try as directory first
    match client.stat(&path).await {
        Ok(entry) if entry.is_dir => {
            let entries = client.list_dir(&path).await?;
            for entry in entries {
                let child_path = format!("{}/{}", path, entry.file_name);
                if entry.is_dir {
                    Box::pin(delete_recursive(client, &child_path)).await?;
                } else {
                    client.remove_file(&child_path).await?;
                }
            }
            client.remove_dir(&path).await?;
            Ok(())
        }
        Ok(_) => {
            // It's a file
            client.remove_file(&path).await
        }
        Err(e) => Err(e),
    }
}

/// Ensure a remote directory exists, creating parent directories as needed.
pub async fn ensure_remote_dir(
    client: &SftpClient,
    path: &str,
) -> Result<(), SftpError> {
    let path = crate::sftp::normalize_path(path);
    if path == "/" || path.is_empty() {
        return Ok(());
    }

    // Check if already exists
    match client.exists(&path).await {
        Ok(true) => return Ok(()),
        _ => {}
    }

    // Create parent first, then self
    if let Some(parent) = path.rsplitn(2, '/').nth(1) {
        let parent = if parent.is_empty() { "/" } else { parent };
        Box::pin(ensure_remote_dir(client, parent)).await?;
    }

    client.create_dir(&path).await
}
