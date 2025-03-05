# SafeCoin Wallet

A secure, cross-platform, multi-coin cryptocurrency wallet built in Rust.

## Project Overview

SafeCoin Wallet is designed with these key principles:

1. **Security**: Private keys are strongly encrypted with minimal attack surfaces
2. **Usability**: Simple interface for beginners, with advanced options for power users
3. **Cross-Platform**: Desktop and mobile support (Windows, macOS, Linux, iOS, Android)
4. **Multi-Coin**: Support for Bitcoin (BTC), Ethereum (ETH), and more

## Features

- **Key Management**
  - Generate and store private keys securely
  - BIP-39 seed phrase generation and recovery
  - AES-256 encryption for stored keys

- **Transaction Handling**
  - Connect to blockchain nodes via APIs
  - Sign transactions offline for enhanced security

- **Cold Storage**
  - Export keys to USB drives
  - Generate paper wallets via QR codes

- **Security Features**
  - Two-factor authentication (2FA)
  - Address validation to prevent errors

## Development Status

This project is currently in active development. See the roadmap below for progress:

- [x] Phase 1: Core Functionality 
  - [x] Key generation and encryption
  - [x] Seed phrase management
  - [x] Basic wallet operations

- [ ] Phase 2: Blockchain Integration
  - [ ] Bitcoin API connection
  - [ ] Ethereum API connection
  - [ ] Transaction signing and broadcasting

- [ ] Phase 3: Security Enhancements
  - [ ] 2FA implementation
  - [ ] Secure storage improvements
  - [ ] Anti-phishing measures

- [ ] Phase 4: UI and Deployment
  - [ ] GUI development with Tauri
  - [ ] Cross-platform testing
  - [ ] Mobile support investigation

## Getting Started

### Prerequisites

- Rust 1.60+ and Cargo

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/safecoin-wallet.git
cd safecoin-wallet

# Build the project
cargo build --release

# Run the wallet
cargo run --release
```

### Testing

```bash
# Run all tests
cargo test

# Run specific test
cargo test wallet_tests
```

## Project Structure

```
safecoin-wallet/
├── src/
│   ├── main.rs          # Entry point
│   ├── wallet.rs        # Core wallet logic
│   ├── blockchain.rs    # Blockchain interaction
│   ├── security.rs      # Authentication & encryption
│   ├── ui.rs            # User interface
│   └── config.rs        # App settings
├── tests/               # Unit and integration tests
├── Cargo.toml           # Rust dependencies
└── README.md            # This file
```

## Security Considerations

- Always back up your seed phrase in a secure location
- Use strong, unique passwords for wallet encryption
- Enable 2FA when available
- Verify recipient addresses before sending transactions
- Consider using a hardware wallet for large amounts

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch: `git checkout -b feature/amazing-feature`
3. Commit your changes: `git commit -m 'Add some amazing feature'`
4. Push to the branch: `git push origin feature/amazing-feature`
5. Open a Pull Request
