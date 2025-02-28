use crate::config::AppConfig;
use bip39::{English, Mnemonic, MnemonicType};
use std::fs::{self, File};
use std::io::{Read, Write};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WalletError {
    #[error("IO error: {0}")]
    IoError(#[form] std::io::Error),

    #[error("Encryption error: {0}")]
    EncryptionError(String),

    #[error("Invalid key error: {0}")]
    InvalidKeyError(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[form] serde_json::Error),

    #[error("Wallet not found: {0}")]
    WalletNotFound(String),

    #[error("Crypto error: {0}")]
    CryptoError(String),
}

pub enum CoinType {
    Bitcoin,
    Ethereum,
    Solana,
    Cardano,
}

pub struct Wallet {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing)]
    #[serde(skip_desrializing)]
    pub seed_phrase: String,
    encrypted_seed: String,
    pub addresses: HashMap<String, String>,
    pub coin_types: Vec<String>,
    pub created_at: u64,
}

impl Wallet {
    pub fn get_addresses(&self, coin_types: CoinType) -> Result<String, WalletError> {
        let coin_type_str = match coin_types {
            CoinType::Bitcoin => "bitcoin",
            CoinType::Ethereum => "ethereum",
            CoinType::Solana => "solana",
            CoinType::Cardano => "cardano",
        };

        match self.addresses.get(coin_type_str) {
            Some(addresses) => Ok(addresses.clone()),
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
        let mnemonic = mnemonic::new(MnemonicType::Words12, Language::English);
        let seed_phrase = mnemonic.phrase().to_string();

        // Derive seed from mnemonic
        let seed = mnemonic.to_seed(password);

        // Generate Wallet addresses for supported coins
        let mut addresses = HashMap::new();

        // Bitcoin address generation
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
    }
}
