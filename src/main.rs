mod blockchain;
mod config;
mod security;
mod ui;
mod wallet;

use blockchain::BlockchainClient;
use config::AppConfig;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Initializing SafeCoin Wallet...");

    // Load configuration
    let config = AppConfig::load()?;

    //Intialize the wallet
    let wallet_manager = wallet::WalletManager::new(&config)?;

    let blockchain_client = BlockchainClient::new(&config.btc_api_url, &config.eth_api_url);

    ui::cli::run_interactive(&wallet_manager, &blockchain_client).await;

    Ok(())
}
