use rand::seq::IndexedRandom;

use crate::wallet::{self, WalletManager};
use crate::blockchain::{BlockchainClient, Transaction, TransactionStatus};
use crate::wallet::{CoinType, Wallet};
use std::collections::HashMap;
use std::io::{self, Read, Write};

pub struct WalletUI {
    wallet_manager: WalletManager,
    blockchain_client: BlockchainClient,
    current_wallet: Option<Wallet>,
}


impl WalletUI {
    pub fn new(wallet_manager: WalletManager, blockchain_client: BlockchainClient) -> Self {
        Self {
            wallet_manager,
            blockchain_client,
            current_wallet: None,
        }
    }

    pub fn run(&mut self) {
        println!("Welccome to SafeCoin Wallet");
        println!("===========================");

        loop {
            if let Err(e) = self.display_main_menu() {
                println!("Error: {}", e);
            }
        }
    }

    fn display_main_menu(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("\nMain Menu");
        println!("-----------");

        match &self.current_wallet {
            Some(wallet) => {
                println!("Current wallet: {} ({})", wallet.name, wallet.id);
                println!("1. View Details");
                println!("2. Check balances");
                println!("3. Send transaction");
                println!("4. Transaction history");
                println!("5. Switch wallet");
                println!("6. Create new wallet");
                println!("7. Settings");
                println!("8. Lock wallet");
                println!("9. Exit");
            },

            None => {
                println!("No wallet loaded");
                println!("1. Create new wallet");
                println!("2. Load existing wallet");
                println!("3. List available wallets");
                println!("4. Import wallets");
                println!("5. Settings");
                println!("6. Exit");
            }
        }

        println!("Enter your choice: ");
        io::stdout().flush();

        let mut choice = String::new();
        io::stdin().read_line(&mut choice)?;
        let choice = choice.trim();

        if self.current_wallet.is_some() {
            match choice {
                "1" => self.view_wallet_details(),
                "2" => self.check_balances().await,
                "3" => self.send_transaction(),
                "4" => self.transaction_history(),
                "5" => self.switch_wallet(),
                "6" => self.create_wallet(),
                "7" => self.settings(),
                "8" => {
                    self.current_wallet = None;
                    println!("Wallet locked successfully");
                },
                "9" => {
                    println!("Thank you for using SafeCoin Wallet!");
                    std::process::exit(0);
                },

                _ => println!("Invalid choice"),
            }
        } else {
            match choice {
                "1" => self.current_wallet,
                "2" => self.load_wallet(),
                "3" => self.list_wallets(),
                "4" => self.impot_wallets(),
                "5" => self.settings(),
                "6" => {
                    println!("Thank you for using SafeCoin Wallet!");
                    std::process::exit(0);
                },
                _ => println!("Invalid choice"),
            }
        }

        Ok(())
    }

    fn view_wallet_details(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(wallet) = &self.current_wallet {
            println!("\nWallet Details");
            println!("----------------");
            println!("Wallet ID: {}", wallet.id);
            println!("Name: {}", wallet.name);
            println!("Created: {}", format_timestamp(wallet.created_at));
            println!("Supported coins");

            for coin_type in wallet.coin_types {
                let address = match coin_type.as_str() {
                    "bitcoin" => wallet.get_address(CoinType::Bitcoin).unwrap_or_default(),
                    "ethereum" => wallet.get_address(CoinType::Ethereum).unwrap_or_default(),
                    "solana" => wallet.get_address(CoinType::Solana).unwrap_or_default(),
                    "cardano" => wallet.get_address(CoinType::Cardano).unwrap_or_default(),
                    _ => "Unknown coin type".to_string(),
                };

                println!(" {} address: {}", capitalize(coin_type), address);
            }
        } else {
            println!("No wallet loaded");
        }

        Ok(())  
    }

    fn check_balances(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(wallet) = &self.current_wallet {
            println!("\nChecking balances...");
            let mut balances = HashMap::new();

            // Check balance for Supported coin types
            for coin_type in &wallet.coin_types {
                match coin_type.as_str() {
                    "bitcoin" => {
                        match self.blockchain_client.get_balance(wallet, CoinType::Bitcoin).await {
                            Ok(balance) => {
                                balances.insert("bitcoin", format!("{:.8} BTC", balance));
                            }, 

                            Err(e) => {
                                balances.insert("bitcoin", format!("Error: {}", e));
                            }
                        }
                    },
                    "ethereum" => {
                        match self.blockchain_client.get_balance(wallet, CoinType::Ethereum).await {
                            Ok(balance) => {
                                balances.insert("ethereum", format!("{:.6} ETH", balance));
                            },
                            Err(e) => {
                                balances.insert("ethereum", format!("Error: {}", e));
                            }
                        }
                    },
                    "solana" => {
                        println!("Solana balance checking not yet implemented");
                    },
                    "cardano" => {
                        println!("Cardano balance checking not yet implemented");
                    },
                    _ => {
                        println!("Unknown coin type: {}", coin_type);
                    }
                }
            }

            println!("\nWallet Balances");
            println!("-----------------");
            for (coin, balance) in balances {
                println!("{}: {}", capitalize(coin), balance);
            }
        } else {
            println!("No wallet  loaded");
        }
        Ok(())
    }


    async fn send_transaction(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(wallet) = &self.current_wallet {
            println!("\nSenf Transaction");
            println!("------------------");

            //Select coin type
            println!("Select coin type:");
            for (i, coin_type) in wallet.coin_types.iter().enumerate() {
                println!("{}: {}", i + 1, capitalize(coin_type));
            }

            println!("Enter your choice (1-{}): ", wallet.coin_types.len());
            io::stdout().flush()?;

            let mut choice = String::new();
            io::stdin().read_line(&mut choice)?;
            let choice = choice.trim().parse::<usize>().unwrap_or(0);

            if choice < 1 || choice > wallet.coin_types.len() {
                println!("Invalid choice");
                return Ok(());
            }

            let coin_type_str = &wallet.coin_types[choice - 1];
            let coin_type = match coin_type_str.as_str() {
                "bitcoin" => CoinType::Bitcoin,
                "ethereum" => CoinType::Ethereum,
                "solana" => CoinType::Solana,
                "cardano" => CoinType::Cardano,
                _ => {
                    println!("Unsupported coin type");
                    return Ok(());
                }
            };

            // Get recipient address
            println!("Enter recipient address: ");
            io::stdout().flush()?;
            let mut to_address = String::new();
            io::stdin().read_line(&mut to_address)?;
            let to_address = to_address.trim();

            // Get amount
            println!("Enter amount to send: ");
            io::stdout().flush()?;
            let mut amount_str = String::new();
            io::stdin().read_line(&mut amount_str)?;
            let amount = match amount_str.trim().parse::<f64>() {
                Ok(a) => a,
                Err(_) => {
                    println!("Invalid amount");
                    return Ok(());
                }
            };

            // Get password
            println!("Enter your wallet password: ");
            io::stdout().flush()?;
            let mut password = String::new();
            io::stdin().read_line(&mut password)?;
            let password = password.trim();

            // Confirm transaction
            println!("\nTransaction Details");
            println!("---------------------");
            println!("Coin: {}", capitalize(coin_type_str));
            println!("From: {}", wallet.get_address(coin_type.clone())?);
            println!("To: {}", to_address);
            println!("Amount: {}", amount);

            println!("Confirm transaction (y/n): ");
            io::stdout().flush()?;
            let mut confirm = String::new();
            io::stdin().read_line(&mut confirm)?;

            if confirm.trim().to_lowercase() != "y" {
                println!("Transaction cancelled");
                return Ok(());
            }

            // Create and broadcast transaction
            match self.blockchain_client.create_transaction(wallet, coin_type, to_address, amount, password).await {
                Ok(transaction) => {
                    println!("Transaction create with ID: {}", transaction.id);
                    match self.blockchain_client.broadcast_transaction(&transaction).await {
                        Ok(tx_id) => {
                            println!("Transaction broadcast successfully!");
                            println!("Transaction ID: {}", tx_id);
                        },
                        Err(e) => {
                            println!("Error broadcasting transaction: {}", e);
                        }
                    }
                },

                Err(e) => {
                    println!("Error creating transaction: {}", e);
                }
            }
        } else {
            println!("No wallet loaded");
        }

        Ok(())
    }

    fn transaction_history(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("\nTransaction History");
        println!("---------------------");
        println!("Feature not yet implemented");

        Ok(())
    }

    fn switch_wallet(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.current_wallet = None;
        self.load_wallet()?;

        Ok(())
    }

    fn create_wallet(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("\nCreate New Wallet");
        println!("-------------------");
        println!("Enter wallet name: ");
        io::stdout().flush()?;
        let mut name = String::new();
        io::stdin().read_line(&mut name)?;
        let name = name.trim();

        println!("Enter password: ");
        io::stdout().flush()?;
        let mut password = String::new();
        io::stdin().read_line(&mut name)?;
        let password = password.trim();

        println!("Confirm password: ");
        io::stdout().flush()?;
        let mut confirm_password = String::new();
        io::stdin().read_line(&mut confirm_password);
        let confirm_password = confirm_password.trim();

        if password != confirm_password {
            println!("Password do not match");
            return Ok(());
        }

        match self.wallet_manager.generate_wallet(name, password) {
            Ok(wallet) => {
                println!("\nWallet created successfully!");
                println!("Wallet ID: {}", wallet.id);
                println!("IMPORTANT: Save your seed phrase seccurely!");
                println!("Seed phrase: {}", wallet.seed_phrase);
                println!("\nPress any key to continue...");

                //Wait for key press
                let mut buffer = [0; 1];
                io::stdin().read_exact(&mut buffer)?;
                self.current_wallet = Some(wallet);
            },
            Err(e) => {
                println!("Error creating wallet: {}", e);
            }
        }

        Ok(())
    }

    fn load_wallet(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("\nLoad Wallet");
        println!("-------------");
        //List available wallets
        let wallets = match self.wallet_manager.list_wallets() {
            Ok(wallets) => wallets,
            Err(e) => {
                println!("Error listing wallets: {}", e);
                return Ok(());
            }
        };

        if wallets.is_empty() {
            println!("No wallets found. Create a new wallet first.");
            return Ok(());
        }

        println!("Available wallets:");
        for (i, wallet_id) in wallets.iter().enumerate() {
            println!("{}, {}", i + 1, wallet_id);
        }

        println!("Enter the number of the wallet to load (1-{}): ", wallets.len());
        io::stdout().flush()?;
        let mut choice = String::new();
        io::stdin().read_line(&mut choice)?;
        let choice = choice.trim().parse::<usize>().unwrap_or(0);

        if choice < 1 || choice > wallets.len() {
            println!("Invalid choice");
            return Ok(());
        }

        let wallet_id = &wallets[choice - 1];

        println!("Enter password: ");
        io::stdout().flush()?;
        let mut password = String::new();
        io::stdin().read_line(&mut password)?;
        let password = password.trim();

        match self.wallet_manager.load_wallet(wallet_id, password) {
            Ok(wallet) {
                println!("Wallet load successfully!");
                self.current_wallet = Some(wallet);
            },
            Err(e) => {
                println!("Error loading wallet: {}", e);
            }
        }

        Ok(())
    }


    fn list_wallets(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("\nAvailable Wallets");
        println!("-------------------");

        match self.wallet_manager.list_wallets() {
            Ok(wallets) => {
                if wallets.is_empty() {
                    println!("No wallets found");
                } else {
                    for (i, wallet_id) in wallets.iter().enumerate() {
                        println!("{}, {}", i + 1, wallet_id);
                    }
                }
            },
            Err(e) => {
                println!("Error listing wallets: {}", e);
            }
        }

        Ok(())
    }

    fn impot_wallets(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("\nImport Wallets");
        println!("----------------");
        println!("Feature not yet implemented");

        Ok(())
    }

    fn settings(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("\nSettings");
        println!("----------");
        println!("Feature not yet implemented");

        Ok(())
    }

}

pub mod cli {
    use super::*;

    pub async fn run_cli(wallet_manager: &WalletManager, blockchain_client: &BlockchainClient) {
        println!("SafeCoin Wallet CLI");
        println!("-------------------");
        
        loop {
            println!("\nMain Menu:");
            println!("1. Create a wallet");
            println!("2. List wallets");
            println!("3. Load wallet");
            println!("4. Check balance");
            println!("5. Send transaction");
            println!("6. Exit");
            print!("Enter your choice: ");
            io::stdout().flush().unwrap();
            
            let mut choice = String::new();
            io::stdin().read_line(&mut choice).unwrap();
            
            match choice.trim() {
                "1" => create_wallet(wallet_manager).await,
                "2" => list_wallets(wallet_manager).await,
                "3" => {
                    let _ = load_wallet(wallet_manager).await; 
                 },
                "4" => check_balance(wallet_manager, blockchain_client).await,
                "5" => send_transaction(wallet_manager, blockchain_client).await,
                "6" => {
                    println!("Thank you for using SafeCoin Wallet!");
                    break;
                },
                _ => println!("Invalid choice, please try again."),
            }
        }
    }
    
    async fn create_wallet(wallet_manager: &WalletManager) {
        println!("\nCreate New Wallet");
        println!("-----------------");
        
        print!("Enter wallet name: ");
        io::stdout().flush().unwrap();
        let mut name = String::new();
        io::stdin().read_line(&mut name).unwrap();
        
        print!("Enter password: ");
        io::stdout().flush().unwrap();
        let mut password = String::new();
        io::stdin().read_line(&mut password).unwrap();
        
        match wallet_manager.generate_wallet(&name.trim(), &password.trim()) {
            Ok(wallet) => {
                println!("\nWallet created successfully!");
                println!("Wallet ID: {}", wallet.id);
                println!("IMPORTANT: Save your seed phrase securely!");
                println!("Seed phrase: {}", wallet.seed_phrase);
            },
            Err(e) => println!("Error creating wallet: {}", e),
        }
    }
    
    async fn list_wallets(wallet_manager: &WalletManager) {
        println!("\nAvailable Wallets");
        println!("-----------------");
        
        match wallet_manager.list_wallets() {
            Ok(wallets) => {
                if wallets.is_empty() {
                    println!("No wallets found");
                } else {
                    for (i, wallet_id) in wallets.iter().enumerate() {
                        println!("{}. {}", i + 1, wallet_id);
                    }
                }
            },
            Err(e) => println!("Error listing wallets: {}", e),
        }
    }
    
    async fn load_wallet(wallet_manager: &WalletManager) -> Option<Wallet> {
        println!("\nLoad Wallet");
        println!("-----------");
        
        print!("Enter wallet ID: ");
        io::stdout().flush().unwrap();
        let mut wallet_id = String::new();
        io::stdin().read_line(&mut wallet_id).unwrap();
        
        print!("Enter password: ");
        io::stdout().flush().unwrap();
        let mut password = String::new();
        io::stdin().read_line(&mut password).unwrap();
        
        match wallet_manager.load_wallet(&wallet_id.trim(), &password.trim()) {
            Ok(wallet) => {
                println!("Wallet loaded successfully!");
                println!("Wallet name: {}", wallet.name);
                
                for coin_type in &wallet.coin_types {
                    match coin_type.as_str() {
                        "bitcoin" => {
                            if let Ok(address) = wallet.get_address(CoinType::Bitcoin) {
                                println!("Bitcoin address: {}", address);
                            }
                        },
                        "ethereum" => {
                            if let Ok(address) = wallet.get_address(CoinType::Ethereum) {
                                println!("Ethereum address: {}", address);
                            }
                        },
                        _ => {}
                    }
                }
                
                Some(wallet)
            },
            Err(e) => {
                println!("Error loading wallet: {}", e);
                None
            }
        }
    }
    
    async fn check_balance(wallet_manager: &WalletManager, blockchain_client: &BlockchainClient) {
        if let Some(wallet) = load_wallet(wallet_manager).await {
            println!("\nChecking balances...");
            
            for coin_type in &wallet.coin_types {
                match coin_type.as_str() {
                    "bitcoin" => {
                        match blockchain_client.get_balance(&wallet, CoinType::Bitcoin).await {
                            Ok(balance) => println!("Bitcoin balance: {} BTC", balance),
                            Err(e) => println!("Error getting Bitcoin balance: {}", e),
                        }
                    },
                    "ethereum" => {
                        match blockchain_client.get_balance(&wallet, CoinType::Ethereum).await {
                            Ok(balance) => println!("Ethereum balance: {} ETH", balance),
                            Err(e) => println!("Error getting Ethereum balance: {}", e),
                        }
                    },
                    _ => println!("{} balance checking not implemented yet", capitalize(coin_type)),
                }
            }
        }
    }
    
    async fn send_transaction(wallet_manager: &WalletManager, blockchain_client: &BlockchainClient) {
        if let Some(wallet) = load_wallet(wallet_manager).await {
            println!("\nSend Transaction");
            println!("-----------------");
            
            // Select coin type
            println!("Select coin type:");
            for (i, coin_type) in wallet.coin_types.iter().enumerate() {
                println!("{}. {}", i + 1, capitalize(coin_type));
            }
            
            print!("Enter choice (1-{}): ", wallet.coin_types.len());
            io::stdout().flush().unwrap();
            let mut choice = String::new();
            io::stdin().read_line(&mut choice).unwrap();
            let choice = choice.trim().parse::<usize>().unwrap_or(0);
            
            if choice < 1 || choice > wallet.coin_types.len() {
                println!("Invalid choice");
                return;
            }
            
            let coin_type_str = &wallet.coin_types[choice - 1];
            let coin_type = match coin_type_str.as_str() {
                "bitcoin" => CoinType::Bitcoin,
                "ethereum" => CoinType::Ethereum,
                "solana" => CoinType::Solana,
                "cardano" => CoinType::Cardano,
                _ => {
                    println!("Unsupported coin type");
                    return;
                }
            };
            
            // Get recipient address
            print!("Enter recipient address: ");
            io::stdout().flush().unwrap();
            let mut to_address = String::new();
            io::stdin().read_line(&mut to_address).unwrap();
            let to_address = to_address.trim();
            
            // Get amount
            print!("Enter amount to send: ");
            io::stdout().flush().unwrap();
            let mut amount_str = String::new();
            io::stdin().read_line(&mut amount_str).unwrap();
            let amount = match amount_str.trim().parse::<f64>() {
                Ok(a) => a,
                Err(_) => {
                    println!("Invalid amount");
                    return;
                }
            };
            
            // Get password
            print!("Enter your wallet password: ");
            io::stdout().flush().unwrap();
            let mut password = String::new();
            io::stdin().read_line(&mut password).unwrap();
            let password = password.trim();
            
            // Create and broadcast transaction
            match blockchain_client.create_transaction(
                &wallet, coin_type, to_address, amount, password
            ).await {
                Ok(transaction) => {
                    println!("Transaction created with ID: {}", transaction.id);
                    
                    println!("Confirm send? (y/n): ");
                    io::stdout().flush().unwrap();
                    let mut confirm = String::new();
                    io::stdin().read_line(&mut confirm).unwrap();
                    
                    if confirm.trim().to_lowercase() == "y" {
                        match blockchain_client.broadcast_transaction(&transaction).await {
                            Ok(tx_id) => {
                                println!("Transaction broadcasted successfully!");
                                println!("Transaction ID: {}", tx_id);
                            },
                            Err(e) => {
                                println!("Error broadcasting transaction: {}", e);
                            }
                        }
                    } else {
                        println!("Transaction cancelled");
                    }
                },
                Err(e) => {
                    println!("Error creating transaction: {}", e);
                }
            }
        }
    }
}

// Helper functions
fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

fn format_timestamp(timestamp: u64) -> String {
    use std::time::{Duration, SystemTime, UNIX_EPOCH};
    use chrono;
    
    let system_time = UNIX_EPOCH + Duration::from_secs(timestamp);
    let datetime = chrono::DateTime::<chrono::Utc>::from(system_time);
    datetime.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}
