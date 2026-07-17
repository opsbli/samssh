//! Known hosts management
//!
//! Loads, verifies, and saves SSH host keys in OpenSSH known_hosts format.
//! File location: `~/.ssh/known_hosts`

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::RwLock;

use russh::keys::key::PublicKey;
use russh::keys::PublicKeyBase64;

/// Thread-safe cache of known host keys.
static KNOWN_HOSTS: once_cell::sync::Lazy<RwLock<HashMap<String, Vec<u8>>>> =
    once_cell::sync::Lazy::new(|| RwLock::new(HashMap::new()));

/// Get the path to the known_hosts file.
fn known_hosts_path() -> PathBuf {
    let home = dirs_next::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".ssh").join("known_hosts")
}

/// Load known hosts from the file into the cache.
///
/// Reads `~/.ssh/known_hosts` and parses each line.
/// Returns the number of hosts loaded.
pub fn load_known_hosts() -> Result<usize, String> {
    let path = known_hosts_path();
    if !path.exists() {
        return Ok(0);
    }

    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;

    let mut cache = KNOWN_HOSTS.write().map_err(|e| e.to_string())?;
    cache.clear();

    let mut count = 0;
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((host, _rest)) = line.split_once(' ') {
            cache.insert(host.to_string(), line.as_bytes().to_vec());
            count += 1;
        }
    }

    Ok(count)
}

/// Verify a host key against the known_hosts cache.
pub fn verify_host_key(host: &str, key: &PublicKey) -> Result<bool, String> {
    let cache = KNOWN_HOSTS.read().map_err(|e| e.to_string())?;

    match cache.get(host) {
        Some(stored_line) => {
            let stored_fingerprint = extract_fingerprint(stored_line);
            let current_fingerprint = key.fingerprint();

            if stored_fingerprint == current_fingerprint {
                Ok(true)
            } else {
                Ok(false)
            }
        }
        None => Err("Host not found in known_hosts".to_string()),
    }
}

/// Save a host key to the known_hosts cache and file.
pub fn save_host_key(host: &str, key: &PublicKey) -> Result<(), String> {
    let fingerprint = key.fingerprint();
    let key_type = key_name(key);
    let key_blob = key_blob_base64(key)?;

    let line = format!("{} {} {} \n", host, key_type, key_blob);
    let path = known_hosts_path();

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create dir {}: {}", parent.display(), e))?;
    }

    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| format!("Failed to open {}: {}", path.display(), e))?;

    file.write_all(line.as_bytes())
        .map_err(|e| format!("Failed to write {}: {}", path.display(), e))?;

    let mut cache = KNOWN_HOSTS.write().map_err(|e| e.to_string())?;
    cache.insert(host.to_string(), line.as_bytes().to_vec());

    tracing::info!("Saved host key for {} (fingerprint: {})", host, fingerprint);
    Ok(())
}

/// Update an existing host key entry.
pub fn update_host_key(host: &str, key: &PublicKey) -> Result<(), String> {
    let fingerprint = key.fingerprint();
    let key_type = key_name(key);
    let key_blob = key_blob_base64(key)?;

    let new_line = format!("{} {} {} \n", host, key_type, key_blob);
    let path = known_hosts_path();

    let content = if path.exists() {
        std::fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?
    } else {
        String::new()
    };

    let mut new_content = String::new();
    let mut replaced = false;
    for line in content.lines() {
        if line.starts_with(host) && line.contains(' ') {
            new_content.push_str(&new_line);
            replaced = true;
        } else {
            new_content.push_str(line);
            new_content.push('\n');
        }
    }
    if !replaced {
        new_content.push_str(&new_line);
    }

    std::fs::write(&path, new_content)
        .map_err(|e| format!("Failed to write {}: {}", path.display(), e))?;

    let mut cache = KNOWN_HOSTS.write().map_err(|e| e.to_string())?;
    cache.insert(host.to_string(), new_line.as_bytes().to_vec());

    tracing::info!("Updated host key for {} (new fingerprint: {})", host, fingerprint);
    Ok(())
}

/// Extract the fingerprint string from a stored known_hosts line.
fn extract_fingerprint(line: &[u8]) -> String {
    let line_str = String::from_utf8_lossy(line);
    let parts: Vec<&str> = line_str.split_whitespace().collect();
    if parts.len() >= 3 {
        parts[2].to_string()
    } else {
        String::new()
    }
}

/// Get a human-readable key type name.
fn key_name(key: &PublicKey) -> String {
    match key {
        PublicKey::Ed25519(_) => "ssh-ed25519".to_string(),
        // In russh-keys, RSA variant uses named fields
        _ => "ssh-rsa".to_string(),
    }
}

/// Get the Base64-encoded key blob for storage.
fn key_blob_base64(key: &PublicKey) -> Result<String, String> {
    Ok(key.public_key_base64())
}

/// Simple Base64 encoding.
fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();

    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
        let b2 = chunk.get(2).copied().unwrap_or(0) as u32;

        let triple = (b0 << 16) | (b1 << 8) | b2;

        result.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        result.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);

        if chunk.len() > 1 {
            result.push(CHARS[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }

        if chunk.len() > 2 {
            result.push(CHARS[(triple & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base64_encode() {
        let input = b"test";
        let output = base64_encode(input);
        assert_eq!(output, "dGVzdA==");
    }

    #[test]
    fn test_base64_encode_empty() {
        let input = b"";
        let output = base64_encode(input);
        assert_eq!(output, "");
    }

    #[test]
    fn test_base64_encode_binary() {
        let input = b"\x00\x01\x02\x03\x04\x05";
        let output = base64_encode(input);
        assert_eq!(output, "AAECAwQF");
    }

    #[test]
    fn test_extract_fingerprint_standard() {
        let line = b"hostname ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAI...";
        let result = extract_fingerprint(line);
        assert_eq!(result, "AAAAC3NzaC1lZDI1NTE5AAAAI...");
    }

    #[test]
    fn test_extract_fingerprint_short_line() {
        let line = b"hostname only";
        let result = extract_fingerprint(line);
        assert_eq!(result, "");
    }

    #[test]
    fn test_extract_fingerprint_empty() {
        let line = b"";
        let result = extract_fingerprint(line);
        assert_eq!(result, "");
    }
}
