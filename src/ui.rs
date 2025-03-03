pub struct WalletUI {}

impl WalletUI {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run() {
        println!("UI not implemented yet. This will be part of phase 4.");
    }
}

pub mod cli {
    use tauri::utils::acl::build::PERMISSION_DOCS_FILE_NAME;

    use crate::blockchain::BlockchainClient;
    use crate::wallet::{CoinType, WalletManager};
    use std::io::{self, Write};

    pub async fn run_cli(wallet_manager: &WalletManager, blockchain_client: &BlockchainClient) {
        println!("SafeCoin Wallet CLI");
        println!("-------------------");
        println!("1. Create an wallet");
        println!("2. Lists wallets");
        println!("3. Load wallet");
        println!("4. Check balance");
        println!("5. Send transaction");
        println!("6: Exit");
        println!("Enter your choice: ");
        io::stdout().flush().unwrap();

        let mut choice = String::new();
        io::stdin().read_line(&mut choice).unwrap();

        match choice.trim() {
            "1" => {
                println!("Enter wallet name: ");
                io::stdout().flush().unwrap();
                let mut name = String::new();
                io::stdin().read_line(&mut name).unwrap();

                println!("Enter password: ");
                io::stdout().flush().unwrap();
                let mut password = String::new();
                io::stdin().read_line(&mut password).unwrap();

                match wallet_manager.generate_wallet(&name.trim(), &password.trim()) {
                    Ok(wallet) => {
                        println!("Wallet created successfully!");
                        println!("Wallet ID: {}", wallet.id);
                        println!("IMPORTANT: Save your seed phrase securely!");
                        println!("Seed phrase: {}", wallet.seed_phrase);
                    }

                    Err(e) => println!("Error creating wallet: {}", e),
                }
            }

            "2" => match wallet_manager.list_wallets() {
                Ok(wallets) => {
                    println!("Wallets:");
                    for (i, wallet_id) in wallets.iter().enumerate() {
                        println!("{}, {}", i + 1, wallet_id);
                    }
                }

                Err(e) => println!("Error listing wallets: {}", e),
            },

            "3" => {
                println!("Enter wallet ID: ");
                io::stdout().flush().unwrap();
                let mut wallet_id = String::new();
                io::stdin().read_line(&mut wallet_id).unwrap();

                println!("Enter password: ");
                io::stdout().flush().unwrap();
                let mut password = String::new();
                io::stdin().read_line(&mut password).unwrap();

                match wallet_manager.load_wallet(&wallet_id.trim(), &password.trim()) {
                    Ok(wallet) => {
                        println!("Wallet loaded successfully!");
                        println!("Wallet name: {}", wallet.name);
                        println!(
                            "Bitcoin address: {}",
                            wallet.get_addresses(CoinType::Bitcoin).unwrap_or_default()
                        );
                        println!(
                            "Ethereum address: {}",
                            wallet.get_addresses(CoinType::Ethereum).unwrap_or_default()
                        );
                    }
                    Err(e) => println!("Error loading wallets: {}", e),
                }
            }

            "4" => {
                println!("Enter wallet ID: ");
                io::stdout().flush().unwrap();
                let mut wallet_id = String::new();
                io::stdin().read_line(&mut wallet_id).unwrap();

                println!("Enter password: ");
                io::stdout().flush().unwrap();
                let mut password = String::new();
                io::stdin().read_line(&mut password).unwrap();

                match wallet_manager.load_wallet(&wallet_id.trim(), &password.trim()) {
                    Ok(wallet) => {
                        println!("Checking balances...");

                        match blockchain_client
                            .get_balance(&wallet, CoinType::Bitcoin)
                            .await
                        {
                            Ok(balance) => println!("Bitcoin balance: {} BTC", balance),
                            Err(e) => println!("Error getting Bitcoin balance: {}", e),
                        }

                        match blockchain_client
                            .get_balance(&wallet, CoinType::Ethereum)
                            .await
                        {
                            Ok(balance) => println!("Ethereum balance: {} ETH", balance),
                            Err(e) => println!("Error getting Ethereum balance: {}", e),
                        }
                    }

                    Err(e) => println!("Error loading wallet: {}", e),
                }
            }

            "5" => {
                println!("Send transaction functionally will be implemented in Phase 2");
            }

            "6" => {}
        }
    }
}
