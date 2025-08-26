# 🚀 Solana DeFi Portfolio Rebalancer

[![Anchor](https://img.shields.io/badge/Anchor-v0.31.1-blue)](https://www.anchor-lang.com/)
[![Solana](https://img.shields.io/badge/Solana-Compatible-green)](https://solana.com/)
[![TypeScript](https://img.shields.io/badge/TypeScript-5.7.3-blue)](https://www.typescriptlang.org/)
[![Tests](https://img.shields.io/badge/Tests-20%2F20%20Passing-brightgreen)](#testing)

A sophisticated **dynamic portfolio rebalancing system** built on Solana that automatically adjusts rebalancing thresholds based on market volatility conditions. The system intelligently manages DeFi strategy allocations to optimize risk-adjusted returns while maintaining mathematical precision and safety.

## 📑 Table of Contents

- [🎯 Features](#-features)
- [🏗️ Architecture](#️-architecture)
- [⚡ Quick Start](#-quick-start)
- [📋 Prerequisites](#-prerequisites)
- [🛠️ Installation](#️-installation)
- [🚀 Usage](#-usage)
- [🧪 Testing](#-testing)
- [📊 Dynamic Threshold System](#-dynamic-threshold-system)
- [🔧 Configuration](#-configuration)
- [📚 Documentation](#-documentation)
- [🤝 Contributing](#-contributing)
- [📄 License](#-license)

## 🎯 Features

### Core Functionality
- **🎯 Dynamic Threshold Rebalancing**: Automatically adjusts rebalancing triggers based on portfolio volatility
- **📊 Performance Scoring**: Mathematical algorithm for ranking DeFi strategies by risk-adjusted returns
- **⚖️ Capital Reallocation**: Intelligent redistribution from underperforming to top-performing strategies
- **🛡️ Risk Management**: Built-in safety mechanisms with overflow protection and bounded calculations
- **⏱️ Time-Based Controls**: Configurable minimum intervals to prevent excessive rebalancing

### Advanced Features
- **📈 Multi-Protocol Support**: Compatible with various DeFi protocols (lending, liquidity providing, yield farming)
- **🔒 Emergency Controls**: Portfolio-wide pause functionality for emergency situations
- **🧮 Mathematical Precision**: Checked arithmetic operations with comprehensive error handling
- **📋 Comprehensive Logging**: Detailed event emission for monitoring and analytics
- **🎚️ Customizable Parameters**: Flexible threshold ranges (10-40%) and interval settings

## 🏗️ Architecture

The system follows a modular architecture with clear separation of concerns:

```
┌─────────────────────────────────────────────────────────────┐
│                    Portfolio Manager                        │
├─────────────────────────────────────────────────────────────┤
│  Dynamic Threshold Engine  │  Performance Scoring Engine   │
├─────────────────────────────────────────────────────────────┤
│  Strategy Registry         │  Capital Allocation Manager   │
├─────────────────────────────────────────────────────────────┤
│              Solana Blockchain Infrastructure               │
└─────────────────────────────────────────────────────────────┘
```

### Key Components
- **Portfolio**: Central account managing strategy collection and rebalancing parameters
- **Strategy**: Individual DeFi protocol integrations with performance tracking
- **Dynamic Threshold Calculator**: Market volatility-responsive threshold adjustment
- **Ranking Engine**: Performance-based strategy evaluation and sorting system

## ⚡ Quick Start

```bash
# Clone the repository
git clone https://github.com/Ansh1902396/portfolio-rebalancer.git
cd portfolio-rebalancer

# Install dependencies
yarn install

# Build the program
anchor build

# Run tests
anchor test

# Deploy to localnet
anchor deploy
```

## 📋 Prerequisites

### Required Software
- **Node.js**: v18.0.0 or higher
- **Yarn**: v1.22.0 or higher  
- **Rust**: v1.70.0 or higher
- **Solana CLI**: v1.18.0 or higher
- **Anchor CLI**: v0.31.1 or higher

### System Requirements
- **OS**: macOS, Linux, or Windows with WSL2
- **Memory**: 8GB RAM minimum (16GB recommended)
- **Storage**: 10GB free space for dependencies and build artifacts

## 🛠️ Installation

### 1. Install Solana Toolchain
```bash
# Install Solana CLI
sh -c "$(curl -sSfL https://release.solana.com/v1.18.12/install)"

# Add to PATH (add to your shell profile)
export PATH="/home/$USER/.local/share/solana/install/active_release/bin:$PATH"

# Verify installation
solana --version
```

### 2. Install Anchor Framework
```bash
# Install Anchor CLI
cargo install --git https://github.com/coral-xyz/anchor anchor-cli --locked

# Verify installation
anchor --version
```

### 3. Configure Solana Environment
```bash
# Set to localnet for development
solana config set --url localhost

# Generate keypair (if you don't have one)
solana-keygen new

# Set as default keypair
solana config set --keypair ~/.config/solana/id.json
```

### 4. Clone and Setup Project
```bash
# Clone repository
git clone https://github.com/Ansh1902396/portfolio-rebalancer.git
cd portfolio-rebalancer

# Install Node.js dependencies
yarn install

# Build Anchor program
anchor build
```

### 5. Start Local Validator
```bash
# In a separate terminal, start local Solana validator
solana-test-validator

# In another terminal, check validator status
solana cluster-version
```

## 🚀 Usage

### Initialize Portfolio
```typescript
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Rebalancer } from "./target/types/rebalancer";

// Connect to program
const program = anchor.workspace.Rebalancer as Program<Rebalancer>;

// Initialize portfolio with dynamic thresholds
await program.methods
  .initializePortfolio(
    managerPublicKey,
    25, // 25% rebalance threshold
    new anchor.BN(3600) // 1 hour minimum interval
  )
  .accounts({
    portfolio: portfolioPda,
    payer: wallet.publicKey,
    manager: managerPublicKey,
    systemProgram: anchor.web3.SystemProgram.programId,
  })
  .rpc();
```

### Register DeFi Strategy
```typescript
// Define protocol configuration
const protocolType = {
  stableLending: {
    poolId: poolPublicKey,
    utilization: 7500, // 75% target utilization
    reserveAddress: reservePublicKey,
  }
};

// Register strategy
await program.methods
  .registerStrategy(
    strategyId,
    protocolType,
    new anchor.BN(1000000000) // 1 SOL initial balance
  )
  .accounts({
    portfolio: portfolioPda,
    strategy: strategyPda,
    manager: managerPublicKey,
    systemProgram: anchor.web3.SystemProgram.programId,
  })
  .rpc();
```

### Execute Rebalancing
```typescript
// Update strategy performance metrics
await program.methods
  .updatePerformance(
    strategyId,
    new anchor.BN(12000), // 120% APY
    2500, // 25% volatility
    new anchor.BN(1200000000) // Updated balance
  )
  .accounts({
    portfolio: portfolioPda,
    strategy: strategyPda,
    manager: managerPublicKey,
  })
  .rpc();

// Execute ranking and rebalancing cycle
await program.methods
  .executeRankingCycle()
  .accounts({
    portfolio: portfolioPda,
    manager: managerPublicKey,
  })
  .rpc();
```

## 🧪 Testing

The project includes comprehensive test suites covering all functionality:

### Run All Tests
```bash
# Execute complete test suite
anchor test

# Run with verbose output
anchor test -- --verbose

# Run specific test file
yarn ts-mocha -p ./tsconfig.json tests/rebalancer.ts
```

### Test Coverage
- **✅ 20/20 Integration Tests Passing**
- **✅ 15/15 Unit Tests Passing** 
- **✅ 4/4 Dynamic Threshold Tests Passing**

### Test Categories
1. **Portfolio Management**: Initialization, configuration, and state management
2. **Strategy Registration**: Protocol integration and validation
3. **Performance Scoring**: Mathematical accuracy and edge cases
4. **Dynamic Thresholds**: Volatility-based threshold adjustment
5. **Rebalancing Logic**: Capital extraction and redistribution
6. **Error Handling**: Comprehensive error scenario testing

## 📊 Dynamic Threshold System

### Core Algorithm
The dynamic threshold system automatically adjusts rebalancing triggers based on portfolio volatility:

```
Dynamic Threshold = Base Threshold + (Average Volatility / 100) × Volatility Multiplier

Where:
- Base Threshold = 15%
- Volatility Multiplier = 20%
- Bounds = [10%, 40%]
```

### Example Scenarios
| Portfolio Volatility | Calculated Threshold | Applied Threshold | Market Condition |
|---------------------|---------------------|-------------------|------------------|
| 5%                  | 16%                 | 16%               | Stable Markets   |
| 15%                 | 18%                 | 18%               | Normal Markets   |
| 30%                 | 21%                 | 21%               | Volatile Markets |
| 50%                 | 25%                 | 25%               | High Volatility  |
| 60%                 | 27%                 | 40% (capped)      | Extreme Volatility |

### Benefits
- **📉 Lower thresholds** in stable markets → More frequent rebalancing → Better optimization
- **📈 Higher thresholds** in volatile markets → Reduced noise trading → Lower transaction costs
- **🛡️ Bounded safety** → Prevents extreme threshold values → System stability

## 🔧 Configuration

### Environment Variables
```bash
# Solana Configuration
export SOLANA_CLUSTER=localnet
export SOLANA_WALLET=~/.config/solana/id.json

# Anchor Configuration  
export ANCHOR_PROVIDER_URL=http://localhost:8899
export ANCHOR_WALLET=~/.config/solana/id.json
```

### Program Constants
```rust
// Threshold bounds
pub const MIN_THRESHOLD: u16 = 1000; // 10%
pub const MAX_THRESHOLD: u16 = 4000; // 40%
pub const BASE_THRESHOLD: u16 = 1500; // 15%

// Time constraints
pub const MIN_REBALANCE_INTERVAL: i64 = 3600; // 1 hour
pub const MAX_REBALANCE_INTERVAL: i64 = 86400; // 24 hours
```

### Customization Options
- **Threshold Bounds**: Adjust min/max values for risk tolerance
- **Base Threshold**: Modify starting point for calculations  
- **Volatility Multiplier**: Change sensitivity to market volatility
- **Rebalance Intervals**: Configure time-based constraints

## 📚 Documentation

Comprehensive documentation is available in the [`docs/`](./docs/) folder:

### Core Documentation
- **[📋 Project Summary](./docs/PROJECT_SUMMARY.md)**: Complete project overview and architecture
- **[🏗️ Account Structures](./docs/ACCOUNT_STRUCTURES_SUMMARY.md)**: Detailed account schema documentation
- **[🎯 Dynamic Threshold Implementation](./docs/DYNAMIC_THRESHOLD_IMPLEMENTATION.md)**: Deep dive into the dynamic threshold system
- **[📊 Mathematical Algorithm Verification](./docs/MATHEMATICAL_ALGORITHM_VERIFICATION.md)**: Mathematical proofs and algorithm analysis

### Testing Documentation  
- **[🧪 Test Results](./docs/COMPREHENSIVE_TEST_RESULTS.md)**: Complete test execution results
- **[🔄 Integration Tests](./docs/COMPREHENSIVE_INTEGRATION_TESTS_SUMMARY.md)**: Integration test coverage and scenarios
- **[📈 Dynamic Threshold Tests](./docs/DYNAMIC_THRESHOLD_SUMMARY.md)**: Specialized dynamic threshold test cases

### Setup Guides
- **[🏁 Portfolio Initialization](./docs/PORTFOLIO_INITIALIZATION_VERIFICATION.md)**: Portfolio setup verification guide
- **[🎯 Strategy Registration](./docs/STRATEGY_REGISTRATION_FINAL_SUCCESS.md)**: Strategy integration guide
- **[📋 General Test Summary](./docs/TEST_SUMMARY.md)**: Overall testing strategy and results

## 🤝 Contributing

We welcome contributions! Please follow these guidelines:

### Development Process
1. **Fork** the repository
2. **Create** a feature branch (`git checkout -b feature/amazing-feature`)
3. **Commit** your changes (`git commit -m 'Add amazing feature'`)
4. **Push** to the branch (`git push origin feature/amazing-feature`)
5. **Open** a Pull Request

### Code Standards
- Follow Rust and TypeScript best practices
- Include comprehensive tests for new features
- Update documentation for API changes
- Ensure all tests pass before submitting

### Bug Reports
Please include:
- Detailed description of the issue
- Steps to reproduce
- Expected vs actual behavior
- Environment details (OS, versions, etc.)

## 📄 License

This project is licensed under the **ISC License** - see the [LICENSE](LICENSE) file for details.

---

## 🚀 Ready to Start?

```bash
# Quick setup for development
git clone https://github.com/Ansh1902396/portfolio-rebalancer.git
cd portfolio-rebalancer
yarn install
anchor build
anchor test
```

**Need help?** Check out our [documentation](./docs/) or open an issue!

**Built with ❤️ on Solana** | **Powered by Anchor Framework** | **Dynamic Thresholds Enabled**
