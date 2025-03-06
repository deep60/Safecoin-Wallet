use crate::blockchain::{BlockchainClient, TransactionStatus};
use crate::wallet::{CoinType, Wallet, WalletManager};
use std::io::{self, Write};

pub async fn run_interactive(wallet_manager: &WalletManager, blockchain_client: &BlockchainClient) {
    println!("Welcome to SafeCoin Wallet CLI");
    println!("==============================");

    let mut current_wallet: Option<Wallet> = None;

    loop {
        println!("\nMain Menu:");
        println!("1. Create new wallet");
        println!("2. Load existing wallet");
        println!("3. List wallets");
        println!("4. Check wallet balance");
        println!("5. Send transaction");
        println!("6. Exit");

        print!("Select an option: ");
        io::stdout().flush().unwrap();

        let mut choice = String::new();
        io::stdin().read_line(&mut choice).unwrap();
        let choice = choice.trim();

        match choice {
            "1" => {
                current_wallet = Some(create_wallet(wallet_manager).unwrap());
            }
            "2" => {
                current_wallet = load_wallet(wallet_manager).await;
            }
            "3" => {
                list_wallets(wallet_manager).await;
            }
            "4" => {
                if let Some(wallet) = &current_wallet {
                    check_balance(wallet, blockchain_client).await;
                } else {
                    println!("Please load a wallet first.");
                }
            }
            "5" => {
                if let Some(wallet) = &current_wallet {
                    send_transaction(wallet, blockchain_client).await;
                } else {
                    println!("Please load a wallet first.");
                }
            }
            "6" => {
                println!("Thank you for using SafeCoin Wallet. Goodbye!");
                break;
            }
            _ => {
                println!("Invalid option. Please try again.");
            }
        }
    }
}

fn create_wallet(wallet_manager: &WalletManager) -> Result<Wallet, Box<dyn std::error::Error>> {
    print!("Enter wallet name: ");
    io::stdout().flush().unwrap();
    let mut name = String::new();
    io::stdin().read_line(&mut name).unwrap();
    let name = name.trim();

    print!("Enter password: ");
    io::stdout().flush().unwrap();
    let mut password = String::new();
    io::stdin().read_line(&mut password).unwrap();
    let password = password.trim();

    let wallet = wallet_manager.generate_wallet(name, password)?;
    println!("Wallet created successfully!");
    println!("Your seed phrase: {}", wallet.seed_phrase);
    println!("IMPORTANT: Write down this seed phrase and keep it safe!");

    Ok(wallet)
}

async fn load_wallet(wallet_manager: &WalletManager) -> Option<Wallet> {
    match wallet_manager.list_wallets() {
        Ok(wallets) if !wallets.is_empty() => {
            println!("Available wallets:");
            for (idx, wallet_id) in wallets.iter().enumerate() {
                println!("{}. {}", idx + 1, wallet_id);
            }

            print!("Select wallet (number): ");
            io::stdout().flush().unwrap();
            let mut choice = String::new();
            io::stdin().read_line(&mut choice).unwrap();
            let choice = choice.trim().parse::<usize>().unwrap_or(0);

            if choice > 0 && choice <= wallets.len() {
                let wallet_id = &wallets[choice - 1];

                print!("Enter password: ");
                io::stdout().flush().unwrap();
                let mut password = String::new();
                io::stdin().read_line(&mut password).unwrap();
                let password = password.trim();

                match wallet_manager.load_wallet(wallet_id, password) {
                    Ok(wallet) => {
                        println!("Wallet loaded successfully!");
                        return Some(wallet);
                    }
                    Err(e) => {
                        println!("Error loading wallet: {}", e);
                        return None;
                    }
                }
            } else {
                println!("Invalid selection.");
                return None;
            }
        }
        Ok(_) => {
            println!("No wallets found. Please create a wallet first.");
            return None;
        }
        Err(e) => {
            println!("Error listing wallets: {}", e);
            return None;
        }
    }
}

async fn list_wallets(wallet_manager: &WalletManager) {
    match wallet_manager.list_wallets() {
        Ok(wallets) => {
            if wallets.is_empty() {
                println!("No wallets found.");
                return;
            }

            println!("Available wallets:");
            for (idx, wallet_id) in wallets.iter().enumerate() {
                println!("{}. {}", idx + 1, wallet_id);
            }
        }
        Err(e) => {
            println!("Error listing wallets: {}", e);
        }
    }
}

async fn check_balance(wallet: &Wallet, blockchain_client: &BlockchainClient) {
    println!("Select coin type:");
    println!("1. Bitcoin");
    println!("2. Ethereum");
    println!("3. Solana");
    println!("4. Cardano");

    print!("Select an option: ");
    io::stdout().flush().unwrap();
    let mut choice = String::new();
    io::stdin().read_line(&mut choice).unwrap();
    let choice = choice.trim();

    let coin_type = match choice {
        "1" => CoinType::Bitcoin,
        "2" => CoinType::Ethereum,
        "3" => CoinType::Solana,
        "4" => CoinType::Cardano,
        _ => {
            println!("Invalid option.");
            return;
        }
    };

    match blockchain_client
        .get_balance(wallet, coin_type.clone())
        .await
    {
        Ok(balance) => {
            let coin_name = match coin_type {
                CoinType::Bitcoin => "BTC",
                CoinType::Ethereum => "ETH",
                CoinType::Solana => "SOL",
                CoinType::Cardano => "ADA",
            };
            println!("Balance: {} {}", balance, coin_name);
        }
        Err(e) => {
            println!("Error fetching balance: {}", e);
        }
    }
}

async fn send_transaction(wallet: &Wallet, blockchain_client: &BlockchainClient) {
    println!("Select coin type:");
    println!("1. Bitcoin");
    println!("2. Ethereum");
    println!("3. Solana");
    println!("4. Cardano");

    print!("Select an option: ");
    io::stdout().flush().unwrap();
    let mut choice = String::new();
    io::stdin().read_line(&mut choice).unwrap();
    let choice = choice.trim();

    let coin_type = match choice {
        "1" => CoinType::Bitcoin,
        "2" => CoinType::Ethereum,
        "3" => CoinType::Solana,
        "4" => CoinType::Cardano,
        _ => {
            println!("Invalid option.");
            return;
        }
    };

    print!("Enter recipient address: ");
    io::stdout().flush().unwrap();
    let mut to_address = String::new();
    io::stdin().read_line(&mut to_address).unwrap();
    let to_address = to_address.trim();

    print!("Enter amount: ");
    io::stdout().flush().unwrap();
    let mut amount_str = String::new();
    io::stdin().read_line(&mut amount_str).unwrap();
    let amount = amount_str.trim().parse::<f64>().unwrap_or(0.0);

    if amount <= 0.0 {
        println!("Invalid amount.");
        return;
    }

    print!("Enter password: ");
    io::stdout().flush().unwrap();
    let mut password = String::new();
    io::stdin().read_line(&mut password).unwrap();
    let password = password.trim();

    match blockchain_client
        .create_transaction(wallet, coin_type.clone(), to_address, amount, password)
        .await
    {
        Ok(transaction) => {
            println!("Transaction created: {}", transaction.id);

            match blockchain_client.broadcast_transaction(&transaction).await {
                Ok(_) => {
                    println!("Transaction broadcast successfully!");
                }
                Err(e) => {
                    println!("Error broadcasting transaction: {}", e);
                    return;
                }
            }

            // Check status
            print!("Checking transaction status...");
            io::stdout().flush().unwrap();

            match blockchain_client
                .get_transaction_status(&transaction.id, coin_type)
                .await
            {
                Ok(status) => match status {
                    TransactionStatus::Pending => {
                        println!("Transaction is pending confirmation.");
                    }
                    TransactionStatus::Confirmed => {
                        println!("Transaction has been confirmed!");
                    }
                    TransactionStatus::Failed => {
                        println!("Transaction failed.");
                    }
                },
                Err(e) => {
                    println!("Error checking transaction status: {}", e);
                }
            }
        }
        Err(e) => {
            println!("Error creating transaction: {}", e);
        }
    }
}
