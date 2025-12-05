# Project Silence - Solana Programs with Arcium

Privacy-preserving AI inference and cross-chain bridge on Solana using Arcium's confidential computing network.

## Overview

Project Silence implements three core components on Solana:

1. **Model Registry** - AI model management with encrypted inference support
2. **Inference Service** - Batch inference processing with user metrics
3. **Silence Bridge** - Intent-based cross-chain bridge with confidential amounts

All confidential computations are executed using Arcium's Multi-Party Execution (MXE) network, ensuring data privacy while maintaining on-chain verifiability.

## Business Use Cases & Problems Solved

### Problems in Current Systems

#### 1. **AI Model Privacy & Trust Issues**

**Problem:**
- AI model providers cannot verify that their proprietary models are executed correctly without exposing model weights
- Users cannot trust that their sensitive prompts (financial data, medical records, personal information) remain private
- No verifiable proof that inference was performed in a trusted execution environment (TEE)
- Model owners have no on-chain record of usage for billing and analytics

**Solution:**
Project Silence enables **privacy-preserving AI inference** where:
- Model weights remain encrypted during execution via Arcium's MXE network
- User prompts are encrypted and never exposed to model providers
- TEE attestation provides cryptographic proof of correct execution
- On-chain metrics track usage without revealing sensitive data

**Use Cases:**
- **Healthcare AI**: Process patient data for diagnosis without exposing PHI
- **Financial AI**: Analyze trading strategies without revealing proprietary algorithms
- **Legal AI**: Review confidential documents with verifiable privacy guarantees
- **Enterprise AI**: Deploy proprietary models with usage tracking and billing

#### 2. **Cross-Chain Bridge Privacy & Trust Gaps**

**Problem:**
- Traditional bridges expose transaction amounts, creating privacy risks
- Users cannot verify that bridge operators execute transfers correctly
- No reputation system for bridge operators (solvers)
- High fees and slow settlement times
- Centralized bridge operators create single points of failure

**Solution:**
Silence Bridge provides an **intent-based, privacy-preserving cross-chain bridge** where:
- Transaction amounts are encrypted using zero-knowledge proofs
- Decentralized solver network with reputation scoring
- Automatic settlement with verifiable execution proofs
- Shielded transfers hide amounts and recipients
- Intent-based model eliminates manual bridging steps

**Use Cases:**
- **Privacy-Conscious DeFi**: Transfer assets between chains without exposing amounts
- **Institutional Trading**: Large cross-chain transfers with privacy protection
- **Multi-Chain Portfolios**: Seamless asset movement across Solana, NEAR, and Zcash
- **Privacy-First Payments**: Cross-chain payments with recipient privacy

#### 3. **Data Privacy in On-Chain Systems**

**Problem:**
- Blockchain transparency conflicts with privacy requirements
- Sensitive business data cannot be processed on-chain
- No way to verify computations without revealing inputs
- Regulatory compliance (GDPR, HIPAA) difficult on public blockchains

**Solution:**
Arcium's confidential computing enables **private on-chain computation**:
- Encrypted data processing in TEE enclaves
- Verifiable computation without data exposure
- Privacy proofs enable auditability without disclosure
- Compliance-friendly architecture for regulated industries

### How Project Silence Works

#### Architecture Overview

Project Silence combines three key technologies:

1. **Arcium MXE Network** - Multi-party execution network that processes encrypted data in TEE enclaves
2. **Solana Blockchain** - Fast, low-cost execution layer for on-chain state management
3. **Zero-Knowledge Proofs** - Cryptographic proofs that verify computations without revealing inputs

#### Solution Flow

**AI Inference Flow:**
```
1. User encrypts prompt → Encrypted prompt hash stored on-chain
2. Inference request created → Payment escrowed, model metadata verified
3. Arcium MXE processes → Encrypted computation in TEE enclaves
4. Result encrypted → Only user can decrypt with their key
5. Attestation verified → Cryptographic proof of correct execution
6. Result stored → Encrypted result hash on-chain, metrics updated
```

**Cross-Chain Bridge Flow:**
```
1. User creates intent → Encrypted amount commitment, recipient hash
2. Funds escrowed → SOL locked in intent vault on Solana
3. Solver matches → Reputation-based matching, stake verification
4. Cross-chain execution → Solver executes transfer on destination chain
5. Execution proof → Destination transaction hash verified
6. Settlement → Automatic reward distribution, protocol fees collected
```

#### Key Innovations

**1. Encrypted Computation**
- All sensitive data processed in encrypted form
- Arcium MXE ensures data never leaves TEE enclaves unencrypted
- Results encrypted for intended recipients only

**2. Verifiable Privacy**
- Zero-knowledge proofs verify computations without revealing inputs
- TEE attestation provides cryptographic guarantees
- On-chain audit trail without data exposure

**3. Decentralized Trust**
- Solver network with reputation scoring
- Staking mechanism ensures honest behavior
- Automatic settlement eliminates manual intervention

**4. Intent-Based Architecture**
- Users specify desired outcome, not execution steps
- Solvers compete to provide best execution
- Automatic matching and settlement

### Business Value

**For AI Model Providers:**
- ✅ Protect proprietary model weights
- ✅ Verify correct execution without exposing models
- ✅ On-chain usage tracking and billing
- ✅ Compliance with data privacy regulations

**For AI Users:**
- ✅ Process sensitive data with privacy guarantees
- ✅ Verify inference correctness via attestation
- ✅ Transparent pricing and usage metrics
- ✅ No trust required in model providers

**For Cross-Chain Users:**
- ✅ Privacy-preserving transfers
- ✅ Lower fees through solver competition
- ✅ Faster settlement with automatic execution
- ✅ No single point of failure

**For Bridge Operators (Solvers):**
- ✅ Earn rewards for providing liquidity
- ✅ Build reputation through successful executions
- ✅ Staking mechanism aligns incentives
- ✅ Transparent fee structure

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

## Deployment Details

### Devnet Deployment

The program is successfully deployed to Solana Devnet (Arcium Testnet).

| Detail | Value |
|--------|-------|
| **Program ID** | `2oFwMgL8qEUN14w6DhJ4jdbccG1FFrosKqH8CVjiN1i2` |
| **Network** | Solana Devnet |
| **IDL Account** | `82n7DhRek69QLAqX7shTTPekZpDwg8j9hRBd7AgmhtrK` |
| **Upgrade Authority** | `HPJqinsnuTFubySxd4kzmom7LpLZDMGd7weqjxvHkvAa` |
| **Program Data Address** | `Ca2MTfsgyYwSuT76jaV2umKJzn6DN6pzbVJsa8eJVxEJ` |
| **Explorer** | [View on Solana Explorer](https://explorer.solana.com/address/2oFwMgL8qEUN14w6DhJ4jdbccG1FFrosKqH8CVjiN1i2/idl?cluster=devnet) |
| **IDL Explorer** | [View IDL](https://explorer.solana.com/address/2oFwMgL8qEUN14w6DhJ4jdbccG1FFrosKqH8CVjiN1i2/idl?cluster=devnet) |

### Deployment Status

✅ **Deployed** - Program is live on devnet  
✅ **IDL Published** - Interface definition available on-chain  
✅ **Arcium Encrypted Instructions** - Built and ready for initialization

### Next Steps

1. Initialize computation definitions using the TypeScript client
2. Run integration tests against devnet
3. Initialize bridge configuration via `initialize_bridge` instruction

## Environment Variables

```env
ANCHOR_PROVIDER_URL=https://api.devnet.solana.com
ANCHOR_WALLET=~/.config/solana/id.json
```

## License

MIT
