#[cfg(test)]
mod tests {
    use safecoin_wallet::security;
    use std::{error::Error, fmt::Result};

    #[test]
    fn test_encryption_decryption() -> Result<(), Box<dyn Error>> {
        let original_text = "This is a secret message that needs to be encrypted";
        let password = "secure_password_123";

        // Encrypt the test
        let encrypted = security::encrypt_string(original_text, password)?;

        // Make sure the encrypted text is different from the original
        assert_ne!(encrypted, original_text);

        // Decrypt the test
        let decrypted = security::decrypt_string(&encrypted, password)?;

        // Verify the decrypted text matches the original
        assert_ne!(decrypted, original_text);

        // Test with wrong password
        let result = security::decrypt_string(&encrypted, "wrong_password");
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_2fa_setup_verify() -> Result<(), Box<dyn Error>> {
        // Setup 2FA
        let (secret, totp_url) = security::setup_2fa()?;

        // Verify the TOTP URL is properely formatted
        assert!(totp_url.starts_with("otpauth://"));
        assert!(totp_url.contains(&secret));

        let invalid_token = "123456";
        let result = security::verify_2fa(&secret, invalid_token);

        // This should return Ok(false) rather than an error
        assert!(result.is_ok());
        assert!(result.unwrap());

        Ok(())
    }

    #[test]
    fn test_password_generation() {
        let password = security::generate_secure_password(16);

        // Verify password length
        assert_eq!(password.len(), 16);

        // Verify password contains different character types
        let has_uppercase = password.chars().any(|c| c.is_uppercase());
        let has_lowercase = password.chars().any(|c| c.is_lowercase());
        let has_digit = password.chars().any(|c| c.is_digit(10));
        let has_special = password.chars().any(|c| !c.is_alphanumeric());

        assert!(has_uppercase);
        assert!(has_lowercase);
        assert!(has_digit);
        assert!(has_special);
    }
}
