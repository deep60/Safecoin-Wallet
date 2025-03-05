use crate::config::AppConfig;
use crate::security;
use bip39::{Language, Mnemonic};
use secp256k1::{PublicKey, Secp256k1, SecretKey};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WalletError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Encryption error: {0}")]
    EncryptionError(String),

    #[error("Invalid key error: {0}")]
    InvalidKeyError(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Wallet not found: {0}")]
    WalletNotFound(String),

    #[error("Crypto error: {0}")]
    CryptoError(String),
}

#[derive(Clone, Debug, PartialEq)]
pub enum CoinType {
    Bitcoin,
    Ethereum,
    Solana,
    Cardano,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Wallet {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub seed_phrase: String,
    encrypted_seed: String,
    pub addresses: HashMap<String, String>,
    pub coin_types: Vec<String>,
    pub created_at: u64,
}

impl Wallet {
    pub fn get_address(&self, coin_type: CoinType) -> Result<String, WalletError> {
        let coin_type_str = match coin_type {
            CoinType::Bitcoin => "bitcoin",
            CoinType::Ethereum => "ethereum",
            CoinType::Solana => "solana",
            CoinType::Cardano => "cardano",
        };

        match self.addresses.get(coin_type_str) {
            Some(address) => Ok(address.clone()),
            None => Err(WalletError::InvalidKeyError(format!(
                "No address for coin type: {}",
                coin_type_str
            ))),
        }
    }
}

pub struct WalletManager {
    config: AppConfig,
    wallets_path: PathBuf,
    secp: Secp256k1<secp256k1::All>,
}

impl WalletManager {
    pub fn new(config: &AppConfig) -> Result<Self, WalletError> {
        let wallets_dir = PathBuf::from(&config.data_dir).join("wallets");
        fs::create_dir_all(&wallets_dir)?;

        Ok(Self {
            config: config.clone(),
            wallets_path: wallets_dir,
            secp: Secp256k1::new(),
        })
    }

    pub fn generate_wallet(&self, name: &str, password: &str) -> Result<Wallet, WalletError> {
        // Generate a random mnemonic (seed phrase)
        let mnemonic = Mnemonic::new(Mnemonic::Words12, Language::English);
        let seed_phrase = mnemonic.phrase().to_string();

        // Derive seed from mnemonic
        let seed = mnemonic.to_seed(password);

        // Generate wallet addresses for supported coins
        let mut addresses = HashMap::new();

        // Bitcoin address generation (simplified for now)
        let bitcoin_secret_key = SecretKey::from_slice(&seed[0..32])
            .map_err(|e| WalletError::CryptoError(e.to_string()))?;
        let bitcoin_public_key = PublicKey::from_secret_key(&self.secp, &bitcoin_secret_key);
        addresses.insert(
            "bitcoin".to_string(),
            format!(
                "btc_{}",
                hex::encode(&bitcoin_public_key.serialize()[0..20])
            ),
        );

        // Ethereum address generation (simplified for now)
        let ethereum_secret_key = SecretKey::from_slice(&seed[32..64])
            .map_err(|e| WalletError::CryptoError(e.to_string()))?;
        let ethereum_public_key = PublicKey::from_secret_key(&self.secp, &ethereum_secret_key);
        addresses.insert(
            "ethereum".to_string(),
            format!("0x{}", hex::encode(&ethereum_public_key.serialize()[1..21])),
        );

        // Encrypt the seed phrase
        let encrypted_seed = security::encrypt_string(&seed_phrase, password)
            .map_err(|e| WalletError::EncryptionError(e.to_string()))?;

        // Create wallet
        let wallet = Wallet {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            seed_phrase: seed_phrase.clone(), // Only available in memory
            encrypted_seed,
            addresses,
            coin_types: vec!["bitcoin".to_string(), "ethereum".to_string()],
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        // Save wallet to disk (encrypted)
        self.save_wallet(&wallet, password)?;

        Ok(wallet)
    }

    fn save_wallet(&self, wallet: &Wallet, password: &str) -> Result<(), WalletError> {
        let wallet_path = self.wallets_path.join(format!("{}.json", wallet.id));

        // Create a copy without the seed phrase for serialization
        let mut wallet_for_storage = wallet.clone();
        wallet_for_storage.seed_phrase = String::new();

        let wallet_json = serde_json::to_string(&wallet_for_storage)?;
        let encrypted_wallet = security::encrypt_string(&wallet_json, password)
            .map_err(|e| WalletError::EncryptionError(e.to_string()))?;

        let mut file = File::create(wallet_path)?;
        file.write_all(encrypted_wallet.as_bytes())?;

        Ok(())
    }

    pub fn load_wallet(&self, wallet_id: &str, password: &str) -> Result<Wallet, WalletError> {
        let wallet_path = self.wallets_path.join(format!("{}.json", wallet_id));

        let mut file = File::open(&wallet_path)
            .map_err(|_| WalletError::WalletNotFound(wallet_id.to_string()))?;

        let mut encrypted_content = String::new();
        file.read_to_string(&mut encrypted_content)?;

        let wallet_json = security::decrypt_string(&encrypted_content, password)
            .map_err(|e| WalletError::EncryptionError(e.to_string()))?;

        let mut wallet: Wallet = serde_json::from_str(&wallet_json)?;

        // Decrypt the seed phrase
        wallet.seed_phrase = security::decrypt_string(&wallet.encrypted_seed, password)
            .map_err(|e| WalletError::EncryptionError(e.to_string()))?;

        Ok(wallet)
    }

    pub fn list_wallets(&self) -> Result<Vec<String>, WalletError> {
        let mut wallet_ids = Vec::new();

        for entry in fs::read_dir(&self.wallets_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                if let Some(filename) = path.file_stem() {
                    if let Some(filename_str) = filename.to_str() {
                        wallet_ids.push(filename_str.to_string());
                    }
                }
            }
        }

        Ok(wallet_ids)
    }
}
