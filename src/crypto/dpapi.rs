//! Windows DPAPI encryption wrapper
//!
//! Provides encrypt/decrypt functions using Windows Data Protection API (DPAPI).
//! Uses `CryptProtectData` and `CryptUnprotectData` with current-user scope.
//! Encrypted data is Base64-encoded for storage in JSON config files.

use std::ptr;

use windows_sys::Win32::Security::Cryptography;
use windows_sys::Win32::Foundation;

/// Error type for crypto operations.
#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("Encryption failed: {0}")]
    EncryptFailed(String),
    #[error("Decryption failed: {0}")]
    DecryptFailed(String),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

/// Encrypt plaintext data using Windows DPAPI.
pub fn encrypt(plaintext: &[u8]) -> Result<Vec<u8>, CryptoError> {
    if plaintext.is_empty() {
        return Err(CryptoError::InvalidInput("Empty data".to_string()));
    }

    let mut plain_blob = Cryptography::CRYPT_INTEGER_BLOB {
        cbData: plaintext.len() as u32,
        pbData: plaintext.as_ptr() as *mut u8,
    };

    let mut cipher_blob = Cryptography::CRYPT_INTEGER_BLOB {
        cbData: 0,
        pbData: ptr::null_mut(),
    };

    let flags = Cryptography::CRYPTPROTECT_UI_FORBIDDEN;
    let result = unsafe {
        Cryptography::CryptProtectData(
            &mut plain_blob,
            ptr::null(),
            ptr::null(),
            ptr::null(),
            ptr::null(),
            flags,
            &mut cipher_blob,
        )
    };

    if result == 0 {
        let err = unsafe { Foundation::GetLastError() };
        return Err(CryptoError::EncryptFailed(format!(
            "DPAPI encrypt failed with error code: {}",
            err
        )));
    }

    let cipher_data = unsafe {
        std::slice::from_raw_parts(cipher_blob.pbData, cipher_blob.cbData as usize).to_vec()
    };

    unsafe { Foundation::LocalFree(cipher_blob.pbData as *mut _) };

    Ok(cipher_data)
}

/// Decrypt ciphertext using Windows DPAPI.
pub fn decrypt(ciphertext: &[u8]) -> Result<Vec<u8>, CryptoError> {
    if ciphertext.is_empty() {
        return Err(CryptoError::InvalidInput("Empty data".to_string()));
    }

    let mut cipher_blob = Cryptography::CRYPT_INTEGER_BLOB {
        cbData: ciphertext.len() as u32,
        pbData: ciphertext.as_ptr() as *mut u8,
    };

    let mut plain_blob = Cryptography::CRYPT_INTEGER_BLOB {
        cbData: 0,
        pbData: ptr::null_mut(),
    };

    let flags = Cryptography::CRYPTPROTECT_UI_FORBIDDEN;
    let result = unsafe {
        Cryptography::CryptUnprotectData(
            &mut cipher_blob,
            ptr::null_mut(),
            ptr::null(),
            ptr::null(),
            ptr::null(),
            flags,
            &mut plain_blob,
        )
    };

    if result == 0 {
        let err = unsafe { Foundation::GetLastError() };
        return Err(CryptoError::DecryptFailed(format!(
            "DPAPI decrypt failed with error code: {}",
            err
        )));
    }

    let plain_data = unsafe {
        std::slice::from_raw_parts(plain_blob.pbData, plain_blob.cbData as usize).to_vec()
    };

    unsafe { Foundation::LocalFree(plain_blob.pbData as *mut _) };

    Ok(plain_data)
}

/// Encrypt and Base64-encode a string for storage.
pub fn encrypt_to_string(plaintext: &str) -> Result<String, CryptoError> {
    let encrypted = encrypt(plaintext.as_bytes())?;
    Ok(base64_encode(&encrypted))
}

/// Base64-decode and decrypt a stored string.
pub fn decrypt_from_string(encoded: &str) -> Result<String, CryptoError> {
    let encrypted = base64_decode(encoded)
        .map_err(|e| CryptoError::InvalidInput(format!("Base64 decode failed: {}", e)))?;
    let decrypted = decrypt(&encrypted)?;
    String::from_utf8(decrypted)
        .map_err(|e| CryptoError::InvalidInput(format!("UTF-8 decode failed: {}", e)))
}

// ── Base64 encoding/decoding ──

const BASE64_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn base64_encode(data: &[u8]) -> String {
    let mut result = String::new();
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
        let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
        let triple = (b0 << 16) | (b1 << 8) | b2;

        result.push(BASE64_CHARS[((triple >> 18) & 0x3F) as usize] as char);
        result.push(BASE64_CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            result.push(BASE64_CHARS[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(BASE64_CHARS[(triple & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}

fn base64_decode(encoded: &str) -> Result<Vec<u8>, String> {
    let encoded = encoded.trim_end_matches('=');
    let mut result = Vec::new();
    let mut buffer = 0u32;
    let mut bits_collected = 0;

    for &byte in encoded.as_bytes() {
        let val = match byte {
            b'A'..=b'Z' => byte - b'A',
            b'a'..=b'z' => byte - b'a' + 26,
            b'0'..=b'9' => byte - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            _ => return Err(format!("Invalid base64 character: {}", byte as char)),
        };
        buffer = (buffer << 6) | val as u32;
        bits_collected += 6;
        if bits_collected >= 8 {
            bits_collected -= 8;
            result.push((buffer >> bits_collected) as u8);
            buffer &= (1 << bits_collected) - 1;
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base64_roundtrip() {
        let data = b"Hello, DPAPI!";
        let encoded = base64_encode(data);
        let decoded = base64_decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_base64_roundtrip_binary() {
        let data = b"\x00\x01\x02\x03\xFF\xFE\xFD\xFC";
        let encoded = base64_encode(data);
        let decoded = base64_decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_base64_roundtrip_empty() {
        let data = b"";
        let encoded = base64_encode(data);
        assert_eq!(encoded, "");
        let decoded = base64_decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_base64_decode_invalid_char() {
        let result = base64_decode("invalid!");
        assert!(result.is_err());
    }

    #[test]
    fn test_base64_padding() {
        let data = b"a";
        let encoded = base64_encode(data);
        assert_eq!(encoded, "YQ==");
        let decoded = base64_decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[cfg(windows)]
    #[test]
    fn test_dpapi_encrypt_decrypt_roundtrip() {
        let plaintext = b"Secret Password!";
        let encrypted = encrypt(plaintext).unwrap();
        let decrypted = decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[cfg(windows)]
    #[test]
    fn test_dpapi_encrypt_empty_fails() {
        let result = encrypt(b"");
        assert!(result.is_err());
    }

    #[cfg(windows)]
    #[test]
    fn test_dpapi_encrypt_decrypt_string_roundtrip() {
        let secret = "MySecretPassword123!";
        let encrypted = encrypt_to_string(secret).unwrap();
        let decrypted = decrypt_from_string(&encrypted).unwrap();
        assert_eq!(decrypted, secret);
    }
}
