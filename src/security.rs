use aes::{
    cipher::{BlockDecrypt, BlockEncrypt, KeyInit},
    Aes256,
};
use hmac::{Hmac, Mac};
use rand::{thread_rng, RngCore};
use sha2::{Digest, Sha256};
use std::error::Error;
use thiserror::Error;
use totp_rs::{Algorithm, Secret, TOTP};

#[derive(Error, Debug)]
pub enum SecurityError {
    #[error("Encryption error: {0}")]
    EncryptionError(String),

    #[error("Decryption error: {0}")]
    DecryptionError(String),

    #[error("Authentication error: {0}")]
    AuthenticationError(String),

    #[error("TOTP error: {0}")]
    TOTPError(String),
}

// Simple encryption - in production, use a more robust approach
pub fn encrypt_string(data: &str, password: &str) -> Result<String, Box<dyn Error>> {
    // Generate a salt
    let mut salt = [0u8; 16];
    thread_rng().fill_bytes(&mut salt);

    // Derive key from password and salt
    let key = derive_key(password, &salt);

    // Create an IV (initialization vector)
    let mut iv = [0u8; 16];
    thread_rng().fill_bytes(&mut iv);

    // Pad the data to be a multiple of 16 bytes (AES block size)
    let mut padded_data = data.as_bytes().to_vec();
    let padding_len = 16 - (padded_data.len() % 16);
    padded_data.extend(vec![padding_len as u8; padding_len]);

    // Encrypt the data
    let cipher = Aes256::new(key.as_slice().into());
    let mut blocks = Vec::new();
    for chunk in padded_data.chunks(16) {
        let mut block = [0u8; 16];
        block.copy_from_slice(chunk);
        cipher.encrypt_block((&mut block).into());
        blocks.extend_from_slice(&block);
    }

    // Combine salt + iv + ciphertext and encode as hex
    let mut result = Vec::new();
    result.extend_from_slice(&salt);
    result.extend_from_slice(&iv);
    result.extend_from_slice(&blocks);

    Ok(hex::encode(result))
}

pub fn decrypt_string(encrypted_hex: &str, password: &str) -> Result<String, Box<dyn Error>> {
    // Decode from hex
    let encrypted_data = hex::decode(encrypted_hex)?;

    if encrypted_data.len() < 32 {
        return Err(Box::new(SecurityError::DecryptionError(
            "Invalid encrypted data".into(),
        )));
    }

    // Extract salt, iv, and ciphertext
    let salt = &encrypted_data[0..16];
    let iv = &encrypted_data[16..32];
    let ciphertext = &encrypted_data[32..];

    // Derive key from password and salt
    let key = derive_key(password, salt);

    // Decrypt the data
    let cipher = Aes256::new(key.as_slice().into());
    let mut blocks = Vec::new();

    for chunk in ciphertext.chunks(16) {
        let mut block = [0u8; 16];
        block.copy_from_slice(chunk);
        cipher.decrypt_block((&mut block).into());
        blocks.extend_from_slice(&block);
    }

    // Remove padding
    if blocks.is_empty() {
        return Err(Box::new(SecurityError::DecryptionError(
            "Decrypted data is empty".into(),
        )));
    }

    let padding_len = blocks[blocks.len() - 1] as usize;
    if padding_len > 0 && padding_len <= 16 {
        let message_len = blocks.len() - padding_len;
        blocks.truncate(message_len);
    } else {
        return Err(Box::new(SecurityError::DecryptionError(
            "Invalid padding".into(),
        )));
    }

    // Convert to string
    let decrypted = String::from_utf8(blocks).map_err(|e| {
        Box::new(SecurityError::DecryptionError(format!(
            "UTF-8 conversion error: {}",
            e
        )))
    })?;
    Ok(decrypted)
}

fn derive_key(password: &str, salt: &[u8]) -> Vec<u8> {
    // Simple key derivation - in production use PBKDF2 or Argon2
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hasher.update(salt);
    hasher.finalize().to_vec()
}

pub fn setup_2fa(username: &str) -> Result<(String, String), Box<dyn std::error::Error>> {
    // Generate a random secret
    let mut secret_bytes = [0u8; 32];
    thread_rng().fill_bytes(&mut secret_bytes);
    let secret = Secret::Raw(secret_bytes.to_vec());
    let base32_secret = secret.to_encoded().to_string();

    // Create TOTP object
    let totp = TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        secret.to_bytes().map_err(|e| Box::new(SecurityError::TOTPError(e.to_string())))?,
    )
    .map_err(|e| Box::new(SecurityError::TOTPError(e.to_string())))?;

    // Generate QR code URL
    let totp_url = format!(
        "otpauth://totp/{}?secret={}&issuer=SafeCoin%20Wallet",
        username, base32_secret
    );

    Ok((base32_secret, totp_url))
}

pub fn verify_2fa(secret: &str, token: &str, username: &str) -> Result<bool, Box<dyn Error>> {
    let secret = Secret::decode_base32(secret)
        .map_err(|e| Box::new(SecurityError::TOTPError(e.to_string())))?;

    let totp = TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        secret.to_bytes().map_err(|e| Box::new(SecurityError::TOTPError(e.to_string())))?,
    )
    .map_err(|e| Box::new(SecurityError::TOTPError(e.to_string())))?;

    Ok(totp
        .check_current(token)
        .map_err(|e| Box::new(SecurityError::TOTPError(e.to_string())))?)
}

// Generate a cryptographically secure random password
pub fn generate_secure_password(length: usize) -> String {
    const CHARSET: &[u8] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*()-_=+";
    let mut rng = thread_rng();

    let password: String = (0..length)
        .map(|_| {
            let idx = rng.next_u32() as usize % CHARSET.len();
            CHARSET[idx] as char
        })
        .collect();

    password
}
