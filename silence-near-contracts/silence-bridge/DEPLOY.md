# Silence Bridge Deployment Guide

## Prerequisites

1. **NEAR CLI installed**: `npm install -g near-cli`
2. **Rust toolchain**: For building the contract
3. **NEAR Testnet account**: You need a NEAR testnet account to deploy

## Step 1: Build the Contract

The contract is already built, but if you need to rebuild:

```bash
cd contracts/silence-bridge
cargo build --target wasm32-unknown-unknown --release
```

The WASM file will be at:
`target/wasm32-unknown-unknown/release/silence_bridge.wasm`

## Step 2: Create Deployment Account

You have two options:

### Option A: Create via Faucet (Recommended for testnet)

```bash
near account create-account sponsor-by-faucet-service silence-bridge.testnet autogenerate-new-keypair save-to-keychain
```

This will:
- Create the account `silence-bridge.testnet`
- Generate a keypair automatically
- Save credentials to your keychain
- Fund the account via the faucet service

### Option B: Create as Subaccount

If you have an existing account (e.g., `your-account.testnet`), create a subaccount:

```bash
near account create-account fund-myself silence-bridge.your-account.testnet autogenerate-new-keypair save-to-keychain
```

Replace `your-account` with your actual NEAR testnet account name.

## Step 3: Deploy the Contract

### Using the Deployment Script

```bash
cd contracts/silence-bridge
./deploy.sh silence-bridge.testnet your-account.testnet
```

Replace:
- `silence-bridge.testnet` with your contract account ID
- `your-account.testnet` with the owner account ID

### Manual Deployment

```bash
cd contracts/silence-bridge

near deploy \
  --wasm-file target/wasm32-unknown-unknown/release/silence_bridge.wasm \
  --init-function new \
  --init-args '{
    "owner_id": "your-account.testnet",
    "min_solver_stake": "1000000000000000000000000",
    "protocol_fee_bps": 50
  }' \
  --network-id testnet \
  silence-bridge.testnet
```

**Parameters:**
- `owner_id`: The account that will own the contract (can execute admin functions)
- `min_solver_stake`: Minimum stake required for solvers (in yoctoNEAR)
  - `1000000000000000000000000` = 1 NEAR
- `protocol_fee_bps`: Protocol fee in basis points
  - `50` = 0.5%
  - `100` = 1%
  - Maximum: `1000` (10%)

## Step 4: Verify Deployment

Check the contract was deployed:

```bash
near view silence-bridge.testnet get_stats --network-id testnet
```

You should see contract statistics including total intents, solvers, and volume.

## Step 5: Update Environment Variables

Add to your `.env.local`:

```env
NEXT_PUBLIC_SILENCE_BRIDGE_CONTRACT=silence-bridge.testnet
```

## Troubleshooting

### Account Already Exists
If the account already exists, you can deploy directly without creating it.

### Insufficient Balance
Make sure the deployment account has enough NEAR for:
- Contract deployment (~3-5 NEAR)
- Storage costs

### Wrong Network
Ensure you're using `--network-id testnet` for testnet deployment.

## Next Steps

After deployment:

1. **Register Solvers**: Solvers need to call `register_solver` with sufficient stake
2. **Create Intents**: Users can create cross-chain intents via `create_intent`
3. **Monitor**: Use `get_stats` and `get_intent` to monitor contract activity

## Contract Methods

See `contracts/README.md` for full documentation of all contract methods.

