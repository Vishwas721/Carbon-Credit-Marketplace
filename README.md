# Carbon Credit Marketplace on Stellar

A decentralized marketplace for trading carbon credits as Stellar tokens with an integrated verification system. Built using Soroban smart contracts on the Stellar blockchain.

## ��� Overview

This project implements a complete carbon credit trading ecosystem that enables:
- **Issuance** of carbon credits with detailed metadata
- **Verification** by authorized verifiers ensuring credit authenticity
- **Trading** on a decentralized marketplace with transparent pricing
- **Retirement** of carbon credits to offset emissions

## ���️ Architecture

The system consists of three main smart contracts:

### 1. Carbon Credit Contract (`carbon-credit`)
Manages the lifecycle of carbon credit tokens with comprehensive metadata including project ID, vintage year, amount in tons, verification status, and issuer information.

**Key Features:** Issue credits, transfer ownership, update verification status, retire credits, query details

### 2. Verification Contract (`verification`)
Provides a robust verification system with role-based access control for authorized verifiers.

**Key Features:** Manage verifier whitelist, submit verification requests, assign verifiers, approve/reject credits

### 3. Marketplace Contract (`marketplace`)
Enables decentralized trading of verified carbon credits with built-in fee mechanism.

**Key Features:** Create listings, buy credits, cancel/update listings, marketplace fee collection (2.5% default)

## ��� Project Structure

```
soroban-hello-world/
├── Cargo.toml
├── README.md
└── contracts/
    ├── carbon-credit/
    │   ├── Cargo.toml
    │   └── src/
    │       ├── lib.rs
    │       └── test.rs
    ├── verification/
    │   ├── Cargo.toml
    │   └── src/
    │       ├── lib.rs
    │       └── test.rs
    └── marketplace/
        ├── Cargo.toml
        └── src/
            ├── lib.rs
            └── test.rs
```

## ��� Getting Started

### Prerequisites

1. **Rust** - Install from [rustup.rs](https://rustup.rs/)
2. **Soroban CLI**: `cargo install --locked soroban-cli`
3. **wasm32 target**: `rustup target add wasm32-unknown-unknown`

### Build & Test

```bash
# Build all contracts
cargo build --release --target wasm32-unknown-unknown

# Run all tests
cargo test

# Test specific contract
cargo test -p carbon-credit
```

## ��� Deployment

### Local Testing

```bash
# Start standalone network
soroban network start standalone

# Deploy contracts
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/carbon_credit.wasm \
  --network standalone
```

### Testnet

```bash
# Configure testnet
soroban network add testnet \
  --rpc-url https://soroban-testnet.stellar.org:443 \
  --network-passphrase "Test SDF Network ; September 2015"

# Generate identity
soroban keys generate --network testnet deployer

# Deploy
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/carbon_credit.wasm \
  --source deployer \
  --network testnet
```

## ��� Usage Examples

### Issue Carbon Credit
```rust
let credit_id = carbon_credit_client.issue_credit(
    &issuer, 
    &String::from_str(&env, "SOLAR-2024-001"),
    &String::from_str(&env, "Solar Farm California"),
    &2024,
    &1000  // tons CO2
);
```

### Verification Workflow
```rust
// Submit verification
verification_client.submit_verification(&issuer, &credit_id, &project_id, &evidence_uri);

// Assign verifier
verification_client.assign_verifier(&admin, &credit_id, &verifier);

// Approve
verification_client.approve_verification(&verifier, &credit_id, &notes);
```

### Marketplace Trading
```rust
// List for sale
let listing_id = marketplace_client.create_listing(&seller, &credit_id, &100, &1000);

// Purchase
marketplace_client.buy_credit(&buyer, &listing_id);

// Retire (offset emissions)
carbon_credit_client.retire_credit(&owner, &credit_id);
```

## ��� Security Features

✅ Authentication required for all state-changing operations  
✅ Role-based access control for verifiers  
✅ Ownership verification for transfers and retirement  
✅ Only verified credits can be retired  
✅ Marketplace fees capped at 10%  

## ��� Key Features

✅ Transparent verification with public audit trail  
✅ Immutable blockchain records  
✅ Decentralized peer-to-peer trading  
✅ Permanent retirement tracking  
✅ Rich metadata (project info, vintage year, tons CO2)  
✅ Market-driven price discovery  

## ��� Contract Events

**Carbon Credit:** `credit_issued`, `verification_updated`, `credit_transferred`, `credit_retired`  
**Verification:** `verifier_added`, `verification_submitted`, `verification_approved`, `verification_rejected`  
**Marketplace:** `listing_created`, `credit_purchased`, `listing_cancelled`, `price_updated`

## ��� Resources

- [Stellar Developers](https://developers.stellar.org/)
- [Soroban Documentation](https://soroban.stellar.org/docs)
- [Soroban Examples](https://github.com/stellar/soroban-examples)

---

**Built with ❤️ using Soroban on Stellar**
# Carbon-Credit-Marketplace
