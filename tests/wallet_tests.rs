#[cfg(test)]
mod tests {
    use safecoin_wallet::config::AppConfig;
    use safecoin_wallet::wallet::{CoinType, WalletManager};
    use std::error::Error;

    #[test]
    fn test_wallet_creation() -> Result<(), Box<dyn Error>> {
        // Create a test-specific config with a temporary directory
        let mut config = AppConfig::default();
        config.data_dir = std::env::temp_dir()
            .join("safecoin-wallet-test")
            .to_string_lossy()
            .into_owned();

        // Initialize wallet manager
        let wallet_manager = WalletManager::new(&config)?;

        // Generate a wallet
        let wallet = wallet_manager.generate_wallet("Test Wallet", "test_password123")?;

        // Verify wallet properties
        assert_eq!(wallet.name, "Test Wallet");
        assert!(!wallet.seed_phrase.is_empty());
        assert!(wallet.get_address(CoinType::Bitcoin).is_ok());
        assert!(wallet.get_address(CoinType::Ethereum).is_ok());

        // Test loading the wallet
        let loaded_wallet = wallet_manager.load_wallet(&wallet.id, "test_password123")?;
        assert_eq!(loaded_wallet.name, wallet.name);
        assert_eq!(loaded_wallet.seed_phrase, wallet.seed_phrase);

        // Test wrong password
        let wrong_pass_result = wallet_manager.load_wallet(&wallet.id, "wrong_password");
        assert!(wrong_pass_result.is_err());

        Ok(())
    }

    #[test]
    fn test_wallet_listing() -> Result<(), Box<dyn Error>> {
        // Create a test-specific config with a temporary directory
        let mut config = AppConfig::default();
        config.data_dir = std::env::temp_dir()
            .join("safecoin-wallet-test-listing")
            .to_string_lossy()
            .into_owned();

        // Initialize wallet manager
        let wallet_manager = WalletManager::new(&config)?;

        // Generate a few wallets
        let wallet1 = wallet_manager.generate_wallet("Wallet 1", "password1")?;
        let wallet2 = wallet_manager.generate_wallet("Wallet 2", "password2")?;

        // List wallets
        let wallets = wallet_manager.list_wallets()?;

        // Veify we have at least the two wallets we created
        assert!(wallets.contains(&wallet1.id));
        assert!(wallets.contains(&wallet2.id));

        Ok(())
    }
}
