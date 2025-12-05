# NEAR Smart Contracts

This directory contains NEAR Protocol smart contracts for AI models, inference workflows, and cross-chain intent-based bridging.

## Contracts

### 1. Model Registry (`near-ai-model-registry`)

Manages AI model registration, metadata, and inference requests/results on-chain.

**Key Features:**
- Register and manage AI models
- Store model metadata (type, version, cost, TEE requirements)
- Create and track inference requests
- Store inference results with TEE attestation
- Verify TEE attestations

**Main Methods:**
- `register_model` - Register a new AI model
- `update_model` - Update model metadata
- `create_inference_request` - Create an inference request
- `store_inference_result` - Store inference result (TEE service only)
- `get_model` - Get model metadata
- `get_all_models` - List all registered models
- `get_request` - Get inference request status
- `get_result` - Get inference result

### 2. Inference Service (`near-ai-inference`)

Handles batch inference, streaming inference, and user metrics.

**Key Features:**
- Batch inference for multiple prompts
- Streaming inference configuration
- User inference metrics tracking
- Integration with model registry

**Main Methods:**
- `create_batch_inference` - Create batch inference request
- `create_streaming_inference` - Create streaming inference session
- `get_batch` - Get batch inference status
- `get_stream` - Get streaming configuration
- `get_user_metrics` - Get user inference statistics

### 3. Silence Bridge (`silence-bridge`)

Intent-based cross-chain bridge registry for seamless transfers between NEAR, Solana, and Zcash.

**Key Features:**
- Intent-based cross-chain execution (no manual bridging)
- Solver network with reputation and staking
- Shielded privacy support via Zcash z-addrs
- Automatic settlement and reward distribution
- Protocol fee mechanism
- Intent lifecycle management (Created → Matched → Executed → Settled)

**Main Methods:**
- `create_intent` - Create a new cross-chain intent (payable)
- `register_solver` - Register as a solver with stake (payable)
- `match_intent` - Match an intent with a solver
- `execute_intent` - Mark intent as executed (solver only)
- `settle_intent` - Settle intent and distribute rewards
- `fail_intent` - Mark intent as failed and refund creator
- `get_intent` - Get intent by ID
- `get_intents_by_creator` - Get all intents created by an account
- `get_solver` - Get solver information
- `get_active_solvers` - List active solvers
- `find_solvers_for_chains` - Find solvers supporting specific chains
- `get_stats` - Get contract statistics

**Intent Lifecycle:**
1. **Created** - User creates intent with deposit
2. **Matched** - Solver matches and commits to execute
3. **Executing** - Cross-chain transfer in progress
4. **Executed** - Transfer completed on destination chain
5. **Settling** - Settlement in progress
6. **Settled** - Fully settled, rewards distributed
7. **Failed** - Execution failed, refund issued
8. **Disputed** - Under dispute resolution

## Building Contracts

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install NEAR CLI
npm install -g near-cli

# Install wasm-pack (for building WASM)
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
```

### Build Commands

```bash
# Build all contracts using the build script
cd contracts
./build.sh

# Or build individually:
# Build Model Registry contract
cd contracts/near-ai-model-registry
cargo build --target wasm32-unknown-unknown --release

# Build Inference Service contract
cd contracts/near-ai-inference
cargo build --target wasm32-unknown-unknown --release

# Build Silence Bridge contract
cd contracts/silence-bridge
cargo build --target wasm32-unknown-unknown --release
```

The compiled WASM files will be in:
- `near-ai-model-registry/target/wasm32-unknown-unknown/release/near_ai_model_registry.wasm`
- `near-ai-inference/target/wasm32-unknown-unknown/release/near_ai_inference.wasm`
- `silence-bridge/target/wasm32-unknown-unknown/release/silence_bridge.wasm`

## Deployment

### Deploy to Testnet

```bash
# Set up NEAR account
near login

# Deploy Model Registry
near deploy --wasmFile target/wasm32-unknown-unknown/release/near_ai_model_registry.wasm \
  --accountId your-account.testnet \
  --initFunction new \
  --initArgs '{"owner_id": "your-account.testnet"}'

# Deploy Inference Service
near deploy --wasmFile target/wasm32-unknown-unknown/release/near_ai_inference.wasm \
  --accountId your-inference-service.testnet \
  --initFunction new \
  --initArgs '{"owner_id": "your-account.testnet", "model_registry_id": "your-model-registry.testnet"}'

# Deploy Silence Bridge
# 
# Step 1: Create the account (choose one method):
#
# Option A: Using faucet service (recommended for testnet)
near account create-account sponsor-by-faucet-service silence-bridge.testnet autogenerate-new-keypair save-to-keychain
#
# Option B: Create as subaccount of your existing account
# Replace 'your-account' with your actual NEAR testnet account
# near account create-account fund-myself silence-bridge.your-account.testnet autogenerate-new-keypair save-to-keychain
#
# Step 2: Build the contract (if not already built)
cd contracts/silence-bridge
cargo build --target wasm32-unknown-unknown --release
#
# Step 3: Deploy the contract
# Note: min_solver_stake is in yoctoNEAR (1 NEAR = 10^24 yoctoNEAR)
# protocol_fee_bps is in basis points (100 = 1%, 50 = 0.5%)
near deploy \
  --wasm-file target/wasm32-unknown-unknown/release/silence_bridge.wasm \
  --init-function new \
  --init-args '{"owner_id": "your-account.testnet", "min_solver_stake": "1000000000000000000000000", "protocol_fee_bps": 50}' \
  --network-id testnet \
  silence-bridge.testnet
#
# Or use the deployment script (handles account check):
./deploy.sh silence-bridge.testnet your-account.testnet
```

### Environment Variables

Add to your `.env.local`:

```env
NEXT_PUBLIC_NEAR_MODEL_REGISTRY_CONTRACT=your-model-registry.testnet
NEXT_PUBLIC_NEAR_INFERENCE_CONTRACT=your-inference-service.testnet
NEXT_PUBLIC_SILENCE_BRIDGE_CONTRACT=silence-bridge.testnet
NEXT_PUBLIC_NEAR_NETWORK_ID=testnet
NEXT_PUBLIC_NEAR_RPC_URL=https://rpc.testnet.near.org
NEXT_PUBLIC_NEAR_WALLET_URL=https://wallet.testnet.near.org
```

## Usage

### AI Contracts

See `lib/near-contract.ts` for TypeScript bindings to interact with AI contracts.

Example:

```typescript
import { createNearContractClient } from "@/lib/near-contract"

const contractClient = createNearContractClient("your-model-registry.testnet")

// Get all models
const models = await contractClient.getAllModels()

// Create inference request
const txHash = await contractClient.createInferenceRequest(
  "req-123",
  "defi-strategy-analyzer",
  "Analyze DeFi strategies",
  undefined,
  true
)
```

### Silence Bridge

See `lib/silence-bridge-contract.ts` and `lib/silence-bridge.ts` for TypeScript bindings.

Example:

```typescript
import { silenceBridgeContract } from "@/lib/silence-bridge-contract"
import { silenceBridge } from "@/lib/silence-bridge"

// Create a cross-chain intent
const intent = await silenceBridge.createIntent({
  destinationChain: "solana",
  destinationAmount: "1000000000", // 1 SOL (in lamports)
  destinationToken: "SOL",
  recipient: "RecipientSolanaAddress...",
  isShielded: false,
  ttlSeconds: 3600
}, "1000000000000000000000000") // 1 NEAR deposit

// Get intent status
const intentStatus = await silenceBridgeContract.getIntent(intent.intentId)

// Register as a solver
await silenceBridgeContract.registerSolver(
  ["Near", "Solana", "Zcash"], // supported chains
  "1000000000000000000000000" // 1 NEAR stake
)
```

## Contract Architecture

### AI Contracts Architecture

```text
┌─────────────────────┐
│  Model Registry     │
│  - Model metadata   │
│  - Inference reqs    │
│  - Results storage   │
└──────────┬──────────┘
           │
           │ references
           │
┌──────────▼──────────┐
│ Inference Service   │
│  - Batch inference  │
│  - Streaming        │
│  - User metrics     │
└─────────────────────┘
```

### Silence Bridge Architecture

```text
┌─────────────────────┐
│   User Intent       │
│   (Cross-chain)     │
└──────────┬──────────┘
           │
           │ creates
           │
┌──────────▼──────────────────┐
│  Silence Bridge Registry    │
│  - Intent lifecycle         │
│  - Solver matching          │
│  - Settlement & rewards     │
└──────────┬──────────────────┘
           │
           │ matches with
           │
┌──────────▼──────────┐
│   Solver Network    │
│  - Staking          │
│  - Reputation       │
│  - Execution        │
└─────────────────────┘
           │
           │ executes
           │
┌──────────▼──────────┐
│  Destination Chain  │
│  (Solana/Zcash)     │
└─────────────────────┘
```

## Security Considerations

### AI Contracts Security

- Only contract owner can register/update models
- Only TEE service can store inference results
- Attestation verification ensures TEE execution
- All inference requests are stored on-chain for auditability
- User metrics are tracked for cost calculation

### Silence Bridge Security

- Solvers must stake minimum amount to participate
- Intent expiration prevents stale orders
- Protocol fees are configurable by owner (max 10%)
- Failed intents automatically refund creators
- Solver reputation system tracks success/failure rates
- Only matched solver can execute or fail an intent
- Settlement is automatic and trustless

