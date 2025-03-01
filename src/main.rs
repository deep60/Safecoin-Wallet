mod blockchain;
mod config;
mod security;
mod ui;
mod wallet;

use config::AppConfig;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Initializing SafeCoin Wallet...");

    // Load configuration
    let config = AppConfig::load()?;

    //Intialize the wallet
    let wallet_manager = wallet::WalletManager::new(&config)?;

    Ok(())
}
