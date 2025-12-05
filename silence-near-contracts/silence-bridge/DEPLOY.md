# Silence Bridge Deployment Guide

## Prerequisites

1. **Docker installed and running** - Required for reproducible WASM builds
2. **NEAR CLI installed**: `npm install -g near-cli-rs`
3. **NEAR Testnet account**: Either existing or will be created via faucet

## Building the Contract

### Important: Use Docker for Reproducible Builds

Due to WASM compatibility requirements with NEAR VM, contracts must be built using Docker to ensure proper WASM format. Local builds with newer Rust versions (1.87+) produce WASM that is incompatible with the NEAR protocol.

```bash
cd silence-near-contracts/silence-bridge

# Build using Docker (recommended)
./docker-build.sh

# Or manually:
docker run --rm \
  -v "$(pwd)":/host \
  -w /host \
  nearprotocol/contract-builder:latest \
  cargo build --target wasm32-unknown-unknown --release
```

## Deployment

### Option 1: Use Deployment Script

```bash
./deploy-testnet.sh <account-id> [owner-id] [min-stake] [fee-bps]

# Example:
./deploy-testnet.sh silence-bridge.testnet
```

### Option 2: Manual Deployment

#### Step 1: Create Account (if needed)

```bash
near account create-account sponsor-by-faucet-service silence-bridge.testnet \
  autogenerate-new-keypair save-to-legacy-keychain network-config testnet
```

#### Step 2: Deploy Contract

```bash
near contract deploy silence-bridge.testnet \
  use-file target/wasm32-unknown-unknown/release/silence_bridge_stripped.wasm \
  with-init-call new \
  json-args '{"owner": "silence-bridge.testnet", "min_solver_stake": "1000000000000000000000000", "protocol_fee_bps": 50}' \
  prepaid-gas '100.0 Tgas' \
  attached-deposit '0 NEAR' \
  network-config testnet \
  sign-with-keychain send
```

### Initialization Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `owner` | AccountId | Contract owner (can call admin methods) |
| `min_solver_stake` | U128 (string) | Minimum stake for solvers in yoctoNEAR |
| `protocol_fee_bps` | u32 | Protocol fee in basis points (50 = 0.5%, max 1000 = 10%) |

## Verification

```bash
# Check contract stats
near view silence-bridge.testnet get_stats --network-id testnet

# Expected output:
{
  "total_intents": 0,
  "total_solvers": 0,
  "active_solvers": 0,
  "total_volume": "0",
  "protocol_fee_bps": 50
}
```

## Post-Deployment

### 1. Register a Solver

```bash
near call silence-bridge.testnet register_solver \
  '{"supported_chains": ["Near", "Solana"]}' \
  --accountId your-solver.testnet \
  --deposit 1 \
  --network-id testnet
```

### 2. Create an Intent

```bash
near call silence-bridge.testnet create_intent \
  '{
    "intent_id": "intent-001",
    "destination_chain": "Solana",
    "destination_amount": "1000000000",
    "destination_token": "SOL",
    "recipient": "SolanaRecipientAddress...",
    "is_shielded": false,
    "ttl_seconds": 3600
  }' \
  --accountId your-account.testnet \
  --deposit 1 \
  --network-id testnet
```

## Environment Variables

Add to your `.env.local`:

```env
NEXT_PUBLIC_SILENCE_BRIDGE_CONTRACT=silence-bridge.testnet
NEXT_PUBLIC_NEAR_NETWORK_ID=testnet
NEXT_PUBLIC_NEAR_RPC_URL=https://rpc.testnet.near.org
```

## Troubleshooting

### CompilationError(PrepareError(Deserialization))

This error indicates the WASM was compiled with an incompatible Rust version. Solution:
- Use Docker builds with `./docker-build.sh`
- Or use Rust 1.86.0 or earlier

### Account Already Exists

If the account exists but has old code, redeploy:
```bash
near contract deploy ... without-init-call ...
```

### Insufficient Balance

Ensure the deployment account has enough NEAR:
- Contract deployment: ~2-3 NEAR
- Storage costs: ~0.1-0.5 NEAR
