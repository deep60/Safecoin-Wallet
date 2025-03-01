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

        // Ethereum address generation
        let ethereum_secret_key = SecretKey::from_slice(&seed[32..64])
            .map_err(|e| WalletError::CryptoError(e.to_string()))?;
        let ethereum_public_key = PublicKey::from_slice_key(&self.secp, &ethereum_secret_key);
        address.insert(
            "ethereum".to_string(),
            format!("0x{}", hex::encode(&ethereum_public_key.serialize()[1..21])),
        );

        let encrypted_seed = security::encrypted_string(&seed_phrase, password)
            .map_err(|e| WalletError::EncryptionError(e.to_string()))?;

        // Create Wallet
        let wallet = Wallet {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            seed_phrase: seed_phrase.clone(), // only available in memory
            encrypted_seed,
            coin_types: vec!["bitcoin".to_string(), "ethereum".to_string()],
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_sec(),
        };

        // Save to wallet to disks (encrypted)
        self.save_wallet(&wallet, password)?;

        Ok(wallet)
    }

    fn save_wallet(&self, wallet: &Wallet, password: &str) -> Result<(), WalletError> {
        let wallet_path = self.wallets_path.join(format!("{}.json", wallet_id));
        //Create a copy without the seed phrase of Serialization
        let mut wallet_for_storage = wallet.clone();
        wallet_for_storage.seed_phrase = String::new();

        let wallet_json = serde_json::to_string(&wallet_for_storage)?;
        let encrypted_wallet = security::encrypted_string(&wallet_json, password)
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
            .map(|e| WalletError::EncryptionError(e.to_string()))?;

        Ok(wallet)
    }

    pub fn list_wallets(&self) -> Result<Vec<String>, WalletError> {
        let mut wallet_ids = Vec::now();

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
