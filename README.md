# Project Silence - Solana Programs with Arcium

Privacy-preserving AI inference and cross-chain bridge on Solana using Arcium's confidential computing network.

## Overview

Project Silence implements three core components on Solana:

1. **Model Registry** - AI model management with encrypted inference support
2. **Inference Service** - Batch inference processing with user metrics
3. **Silence Bridge** - Intent-based cross-chain bridge with confidential amounts

All confidential computations are executed using Arcium's Multi-Party Execution (MXE) network, ensuring data privacy while maintaining on-chain verifiability.

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        Project Silence on Solana                        │
├──────────────────┬────────────────────┬─────────────────────────────────┤
│  Model Registry  │  Inference Service │       Silence Bridge            │
│  - Register AI   │  - Batch inference │  - Create intents               │
│  - Model metadata│  - User metrics    │  - Solver registration          │
│  - TEE required  │  - Cost tracking   │  - Match & execute              │
│                  │                    │  - Settlement & rewards         │
└────────┬─────────┴─────────┬──────────┴──────────────┬──────────────────┘
         │                   │                         │
         └───────────────────┴─────────────────────────┘
                             │
         ┌───────────────────┴───────────────────┐
         │          Arcium MXE Network           │
         │  - Encrypted inference processing     │
         │  - Intent amount verification         │
         │  - Privacy proof generation           │
         │  - Reputation score calculation       │
         │  - TEE attestation verification       │
         └───────────────────────────────────────┘
```

## Programs

### 1. Model Registry

Manages AI model registration and inference requests on Solana.

**Key Features:**
- Register and manage AI models with metadata
- Create inference requests with encrypted prompts
- Store results with TEE attestation verification
- Track per-user inference metrics and costs

**Instructions:**
- `register_model` - Register a new AI model
- `update_model` - Update model metadata
- `create_inference_request` - Create an inference request
- `process_inference` - Queue encrypted inference computation
- `store_inference_result` - Store inference result (TEE service)

### 2. Inference Service

Handles batch inference and streaming inference requests.

**Key Features:**
- Batch inference for multiple prompts (up to 100)
- User inference metrics tracking
- Integration with model registry
- Cost calculation and payment

**Instructions:**
- `create_batch_inference` - Create batch inference request

### 3. Silence Bridge

Intent-based cross-chain bridge for seamless transfers between Solana, NEAR, and Zcash.

**Key Features:**
- Intent-based cross-chain execution (no manual bridging)
- Solver network with reputation and staking
- Shielded privacy support via encrypted amounts
- Automatic settlement and reward distribution
- Protocol fee mechanism

**Intent Lifecycle:**
1. **Created** - User creates intent with deposit
2. **Matched** - Solver matches and commits to execute
3. **Executing** - Cross-chain transfer in progress
4. **Executed** - Transfer completed on destination chain
5. **Settled** - Fully settled, rewards distributed
6. **Failed** - Execution failed, refund issued

**Instructions:**
- `initialize_bridge` - Initialize bridge configuration
- `register_solver` - Register as a solver with stake
- `create_intent` - Create a cross-chain intent
- `match_intent` - Match an intent with a solver
- `execute_intent` - Mark intent as executed
- `settle_intent` - Settle intent and distribute rewards
- `fail_intent` - Mark intent as failed and refund creator

## Encrypted Instructions (Arcium)

Located in `encrypted-ixs/`, these define confidential computations executed via Arcium MXE:

| Instruction | Purpose |
|-------------|---------|
| `process_inference` | Process encrypted inference input data |
| `verify_intent_amounts` | Verify encrypted source/destination amounts |
| `compute_settlement` | Calculate encrypted settlement distribution |
| `calculate_reputation` | Calculate encrypted solver reputation score |
| `verify_attestation` | Verify TEE attestation in encrypted domain |
| `generate_privacy_proof` | Generate privacy proof for shielded transfers |

## Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Solana CLI
sh -c "$(curl -sSfL https://release.anza.xyz/stable/install)"

# Install Anchor
cargo install --git https://github.com/coral-xyz/anchor avm --locked
avm install latest
avm use latest

# Install Arcium CLI
npm install -g arcium
```

## Build & Deploy

### Build

```bash
cd project_silence

# Build Anchor program
anchor build

# Build encrypted instructions (Arcium)
arcium build

# Or use the combined script
yarn build
```

### Test

```bash
# Run tests
anchor test

# Or with yarn
yarn test
```

### Deploy to Devnet

```bash
# Set cluster
solana config set --url devnet

# Deploy program
anchor deploy --provider.cluster devnet

# Initialize computation definitions (required for Arcium)
# This is done via TypeScript client after deployment
```

## Account Structures

### ModelMetadata
```rust
pub struct ModelMetadata {
    pub model_id: u64,
    pub name: String,          // max 32 chars
    pub description: String,   // max 256 chars
    pub model_type: u8,        // 0=LLM, 1=Embedding, 2=Classifier, 3=Other
    pub version: String,       // max 16 chars
    pub owner: Pubkey,
    pub tee_required: bool,
    pub attestation_required: bool,
    pub cost_per_inference: u64,
    pub is_active: bool,
    // ... timestamps and bump
}
```

### Intent
```rust
pub struct Intent {
    pub intent_id: u64,
    pub creator: Pubkey,
    pub source_chain: Chain,           // Solana
    pub destination_chain: Chain,      // Near, Zcash
    pub source_amount: u64,            // lamports
    pub destination_amount_commitment: [u8; 32],  // encrypted
    pub recipient_hash: [u8; 32],      // for privacy
    pub is_shielded: bool,
    pub status: IntentStatus,
    pub solver: Option<Pubkey>,
    // ... timestamps, proofs, bump
}
```

### Solver
```rust
pub struct Solver {
    pub solver_id: Pubkey,
    pub supported_chains: u8,  // bitmap
    pub stake: u64,
    pub reputation_score: u32, // 0-1000
    pub total_intents_executed: u64,
    pub successful_intents: u64,
    pub failed_intents: u64,
    pub total_volume: u64,
    pub is_active: bool,
    // ... timestamps, bump
}
```

## TypeScript Integration

```typescript
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { ProjectSilence } from "../target/types/project_silence";

// Initialize provider
const provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);

// Get program
const program = anchor.workspace.ProjectSilence as Program<ProjectSilence>;

// Register a model
await program.methods
  .registerModel(
    new anchor.BN(1),                    // model_id
    "GPT-4 Agent",                       // name
    "AI model for DeFi strategies",      // description
    0,                                   // model_type (LLM)
    "1.0.0",                             // version
    true,                                // tee_required
    true,                                // attestation_required
    new anchor.BN(100000)                // cost_per_inference (lamports)
  )
  .accounts({
    owner: provider.wallet.publicKey,
    model: modelPda,
    systemProgram: anchor.web3.SystemProgram.programId,
  })
  .rpc();

// Create a cross-chain intent
await program.methods
  .createIntent(
    new anchor.BN(1),                    // intent_id
    { near: {} },                        // destination_chain
    destinationAmountCommitment,         // encrypted
    destinationTokenHash,
    recipientHash,
    true,                                // is_shielded
    new anchor.BN(3600)                  // ttl_seconds
  )
  .accounts({
    creator: provider.wallet.publicKey,
    config: configPda,
    intent: intentPda,
    deposit: depositAccount,
    systemProgram: anchor.web3.SystemProgram.programId,
  })
  .rpc();
```

## Security Considerations

### Model Registry
- Only model owner can update model metadata
- Inference requests require payment equal to model cost
- Results require TEE attestation when model requires it
- All metrics are tracked for auditing

### Silence Bridge
- Solvers must stake minimum amount to participate
- Intent expiration prevents stale orders
- Protocol fees are capped at 10%
- Failed intents automatically refund creators
- Solver reputation tracks success/failure rates
- Only matched solver can execute or fail an intent
- Settlement is automatic and trustless

### Arcium Integration
- All confidential data is processed via MXE
- Encrypted computations never expose plaintext values
- Results are encrypted for intended observers only
- Privacy proofs enable verification without disclosure

## Program IDs

| Network | Program ID |
|---------|------------|
| Localnet | `2oFwMgL8qEUN14w6DhJ4jdbccG1FFrosKqH8CVjiN1i2` |
| Devnet | `2oFwMgL8qEUN14w6DhJ4jdbccG1FFrosKqH8CVjiN1i2` |

## Environment Variables

```env
ANCHOR_PROVIDER_URL=https://api.devnet.solana.com
ANCHOR_WALLET=~/.config/solana/id.json
```

## License

MIT
