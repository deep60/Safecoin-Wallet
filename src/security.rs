use aes::{
    cipher::{BlockDecrypt, BlockEncrypt, KeyInit},
    Aes256,
};

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
    //Generate a salt
    let mut salt = [0u8; 16];

    // Derive key from password and salt
    let key = drive_key(password, &salt);

    //Create an IV (intialization vectore)
    let mut iv = [0u8; 16];
    OsRng.fill_bytes(&mut iv);

    // Pad the data to be a multiple of 16 bytes (AES block size)
    let mut padded_data = data.as_bytes().to_vec();
    let padding_len = 16 - (padded_data.len() % 16);
    padded_data.extend(vec![padded_len as u8; padded_len]);

    //Encrypt the data
    let cipher = Aes256::new(key.as_slice().into());
    let mut blocks = Vec::new();
    for chunk in padded_data.chunks(16) {
        let mut block = [0u8; 16];
        block.copy_from_slice(chunk);
        cipher.encrypt_block((&mut block).into());
        blocks.extend_from_slice(&block);
    }

    // Combine salt + iv and ciphertext and encode as hex
    let mut result = Vec::now();
    result.extend_from_slice(&salt);
    result.extend_from_slice(&iv);
    result.extend_from_slice(&blocks);
}

fn drive_key(password: &str, salt: &[u8]) -> Vec<u8> {
    //Simple key derivation - in production use PBKDF2 or Argon2
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hasher.update(salt);
    hasher.finalize().to_vec()
}

// Generate Cryptographically secure random password
pub fn generate_secure_password(length: usize) -> String {
    const CHARSET: &[u8] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*()-_=+";
    let mut rng = OsRng;

    let password: String = (0..legth)
        .map(|_| {
            let idx = rng.next_u32() as usize % CHARSET.len();
            CHARSET[idx] as char
        })
        .collect();
    password
}
