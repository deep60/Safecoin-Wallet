use crate::wallet::{CoinType, Wallet, WalletError};
use serde::{Deserialize, Serialize};
use std::error::Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BlockchainError {
    #[error("API error: {0}")]
    ApiError(String),

    #[error("Transaction error: {0}")]
    TransactionError(String),

    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("Wallet error: {0}")]
    WalletError(#[from] WalletError),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Transaction {
    pub id: String,
    pub wallet_id: String,
    pub coin_type: String,
    pub from_address: String,
    pub to_address: String,
    pub amount: f64,
    pub fee: f64,
    pub status: TransactionStatus,
    pub timestamp: u64,
    pub block_height: Option<u64>,
    pub confirmations: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum TransactionStatus {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "confirmed")]
    Confirmed,
    #[serde(rename = "failed")]
    Failed,
}

pub struct BlockchainClient {
    btc_api_url: String,
    eth_api_url: String,
    sol_api_url: String,
    ada_api_url: String,
    http_client: reqwest::Client,
}

impl BlockchainClient {
    pub fn new(btc_api_url: &str, eth_api_url: &str) -> Self {
        Self {
            btc_api_url: btc_api_url.to_string(),
            eth_api_url: eth_api_url.to_string(),
            sol_api_url: "https://api.solana.com".to_string(), // Default URL
            ada_api_url: "https://api.cardano.org".to_string(), // Default URL
            http_client: reqwest::Client::new(),
        }
    }

    pub async fn get_balance(
        &self,
        wallet: &Wallet,
        coin_type: CoinType,
    ) -> Result<f64, BlockchainError> {
        let address = wallet.get_address(coin_type.clone())?;

        match coin_type {
            CoinType::Bitcoin => self.get_btc_balance(&address).await,
            CoinType::Ethereum => self.get_eth_balance(&address).await,
            CoinType::Solana => self.get_sol_balance(&address).await,
            CoinType::Cardano => self.get_ada_balance(&address).await,
        }
    }

    async fn get_btc_balance(&self, address: &str) -> Result<f64, BlockchainError> {
        println!("Querying BTC balance for address: {}", address);
        Ok(0.01234)
    }

    async fn get_eth_balance(&self, address: &str) -> Result<f64, BlockchainError> {
        println!("Querying ETH balance for address: {}", address);
        Ok(1.5678)
    }

    async fn get_sol_balance(&self, address: &str) -> Result<f64, BlockchainError> {
        println!("Querying SOL balance for address: {}", address);
        Ok(2.3456)
    }

    async fn get_ada_balance(&self, address: &str) -> Result<f64, BlockchainError> {
        println!("Querying ADA balance for address: {}", address);
        Ok(100.5678)
    }

    pub async fn create_transaction(
        &self,
        wallet: &Wallet,
        coin_type: CoinType,
        to_address: &str,
        amount: f64,
        password: &str,
    ) -> Result<Transaction, BlockchainError> {
        let from_address = wallet.get_address(coin_type.clone())?;
        let coin_type_str = match coin_type {
            CoinType::Bitcoin => "bitcoin",
            CoinType::Ethereum => "ethereum",
            CoinType::Solana => "solana",
            CoinType::Cardano => "cardano",
        };

        let transaction = Transaction {
            id: uuid::Uuid::new_v4().to_string(),
            wallet_id: wallet.id.clone(),
            coin_type: coin_type_str.to_string(),
            from_address,
            to_address: to_address.to_string(),
            amount,
            fee: 0.0001,
            status: TransactionStatus::Pending,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            block_height: None,
            confirmations: None,
        };

        Ok(transaction)
    }

    pub async fn broadcast_transaction(
        &self,
        transaction: &Transaction,
    ) -> Result<String, BlockchainError> {
        println!("Broadcasting transaction: {:?}", transaction);
        Ok(transaction.id.clone())
    }

    pub async fn get_transaction_status(
        &self,
        transaction_id: &str,
        coin_type: CoinType,
    ) -> Result<TransactionStatus, BlockchainError> {
        println!("Checking status of transaction: {}", transaction_id);
        Ok(TransactionStatus::Confirmed)
    }
}
