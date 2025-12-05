use anchor_lang::prelude::*;
use arcium_anchor::prelude::*;

// Computation definition offsets for encrypted instructions
const COMP_DEF_OFFSET_PROCESS_INFERENCE: u32 = comp_def_offset("process_inference");
const COMP_DEF_OFFSET_VERIFY_INTENT_AMOUNTS: u32 = comp_def_offset("verify_intent_amounts");
const COMP_DEF_OFFSET_COMPUTE_SETTLEMENT: u32 = comp_def_offset("compute_settlement");
const COMP_DEF_OFFSET_CALCULATE_REPUTATION: u32 = comp_def_offset("calculate_reputation");
const COMP_DEF_OFFSET_VERIFY_ATTESTATION: u32 = comp_def_offset("verify_attestation");
const COMP_DEF_OFFSET_GENERATE_PRIVACY_PROOF: u32 = comp_def_offset("generate_privacy_proof");

declare_id!("2oFwMgL8qEUN14w6DhJ4jdbccG1FFrosKqH8CVjiN1i2");

// ============================================================================
// STATE ACCOUNTS
// ============================================================================

/// Model metadata stored on-chain
#[account]
#[derive(InitSpace)]
pub struct ModelMetadata {
    /// Unique model identifier
    pub model_id: u64,
    /// Model name (max 32 chars)
    #[max_len(32)]
    pub name: String,
    /// Model description (max 256 chars)
    #[max_len(256)]
    pub description: String,
    /// Model type: 0=LLM, 1=Embedding, 2=Classifier, 3=Other
    pub model_type: u8,
    /// Version string (max 16 chars)
    #[max_len(16)]
    pub version: String,
    /// Owner authority
    pub owner: Pubkey,
    /// Whether TEE execution is required
    pub tee_required: bool,
    /// Whether attestation is required
    pub attestation_required: bool,
    /// Cost per inference in lamports
    pub cost_per_inference: u64,
    /// Creation timestamp
    pub created_at: i64,
    /// Last update timestamp
    pub updated_at: i64,
    /// Whether model is active
    pub is_active: bool,
    /// Bump for PDA derivation
    pub bump: u8,
}

/// Inference request stored on-chain
#[account]
#[derive(InitSpace)]
pub struct InferenceRequest {
    /// Request ID (unique)
    pub request_id: u64,
    /// Associated model ID
    pub model_id: u64,
    /// User who created request
    pub user: Pubkey,
    /// Prompt hash (for privacy - actual prompt encrypted)
    pub prompt_hash: [u8; 32],
    /// Whether attestation is required
    pub require_attestation: bool,
    /// Creation timestamp
    pub created_at: i64,
    /// Status: 0=Pending, 1=Processing, 2=Completed, 3=Failed
    pub status: u8,
    /// Result hash (encrypted)
    pub result_hash: [u8; 32],
    /// TEE attestation proof hash
    pub attestation_hash: [u8; 32],
    /// Bump for PDA derivation
    pub bump: u8,
}

/// Batch inference request
#[account]
#[derive(InitSpace)]
pub struct BatchInference {
    /// Batch ID
    pub batch_id: u64,
    /// Associated model ID
    pub model_id: u64,
    /// Number of prompts in batch
    pub prompt_count: u32,
    /// User who created batch
    pub user: Pubkey,
    /// Whether attestation is required
    pub require_attestation: bool,
    /// Creation timestamp
    pub created_at: i64,
    /// Completed count
    pub completed_count: u32,
    /// Failed count
    pub failed_count: u32,
    /// Bump for PDA derivation
    pub bump: u8,
}

/// User inference metrics
#[account]
#[derive(InitSpace)]
pub struct UserMetrics {
    /// User pubkey
    pub user: Pubkey,
    /// Total inference requests
    pub total_inferences: u64,
    /// Successful inferences
    pub successful_inferences: u64,
    /// Failed inferences
    pub failed_inferences: u64,
    /// Total cost paid in lamports
    pub total_cost: u64,
    /// Average latency in milliseconds
    pub average_latency_ms: u64,
    /// Bump for PDA derivation
    pub bump: u8,
}

/// Supported blockchain networks for bridging
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace)]
pub enum Chain {
    Solana,
    Near,
    Zcash,
}

/// Intent status lifecycle
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace)]
pub enum IntentStatus {
    Created,    // Intent created, awaiting solver
    Matched,    // Solver matched, preparing execution
    Executing,  // Cross-chain transfer in progress
    Executed,   // Transfer completed on destination
    Settling,   // Settlement in progress
    Settled,    // Fully settled, rewards distributed
    Failed,     // Execution failed
    Disputed,   // Under dispute resolution
}

/// Cross-chain transfer intent
#[account]
#[derive(InitSpace)]
pub struct Intent {
    /// Unique intent ID
    pub intent_id: u64,
    /// Creator of the intent
    pub creator: Pubkey,
    /// Source chain (always Solana for this program)
    pub source_chain: Chain,
    /// Destination chain
    pub destination_chain: Chain,
    /// Source amount in lamports
    pub source_amount: u64,
    /// Destination amount (encrypted commitment)
    pub destination_amount_commitment: [u8; 32],
    /// Source token (SPL token mint or native SOL)
    pub source_token: Pubkey,
    /// Destination token identifier hash
    pub destination_token_hash: [u8; 32],
    /// Recipient address hash (for privacy)
    pub recipient_hash: [u8; 32],
    /// Whether this is a shielded transfer
    pub is_shielded: bool,
    /// Current status
    pub status: IntentStatus,
    /// Matched solver (if any)
    pub solver: Option<Pubkey>,
    /// Creation timestamp
    pub created_at: i64,
    /// Expiration timestamp
    pub expires_at: i64,
    /// Execution timestamp (if executed)
    pub executed_at: Option<i64>,
    /// Destination transaction hash
    pub destination_tx_hash: [u8; 32],
    /// Privacy proof (for shielded transfers)
    pub privacy_proof: [u8; 32],
    /// Bump for PDA derivation
    pub bump: u8,
}

/// Solver entity
#[account]
#[derive(InitSpace)]
pub struct Solver {
    /// Solver authority
    pub solver_id: Pubkey,
    /// Supported chains bitmap (bit 0=Solana, 1=Near, 2=Zcash)
    pub supported_chains: u8,
    /// Staked amount in lamports
    pub stake: u64,
    /// Reputation score (0-1000)
    pub reputation_score: u32,
    /// Total intents executed
    pub total_intents_executed: u64,
    /// Successful intents
    pub successful_intents: u64,
    /// Failed intents
    pub failed_intents: u64,
    /// Total volume processed in lamports
    pub total_volume: u64,
    /// Whether solver is active
    pub is_active: bool,
    /// Registration timestamp
    pub registered_at: i64,
    /// Bump for PDA derivation
    pub bump: u8,
}

/// Bridge configuration
#[account]
#[derive(InitSpace)]
pub struct BridgeConfig {
    /// Contract owner/admin
    pub owner: Pubkey,
    /// Minimum solver stake in lamports
    pub min_solver_stake: u64,
    /// Protocol fee in basis points (100 = 1%)
    pub protocol_fee_bps: u16,
    /// Total volume processed
    pub total_volume: u64,
    /// Total intents created
    pub total_intents: u64,
    /// Total active solvers
    pub active_solvers: u32,
    /// Protocol fee vault
    pub fee_vault: Pubkey,
    /// Bump for PDA derivation
    pub bump: u8,
}

// ============================================================================
// MAIN PROGRAM
// ============================================================================

#[arcium_program]
pub mod project_silence {
    use super::*;

    // ========================================================================
    // INITIALIZATION INSTRUCTIONS
    // ========================================================================

    /// Initialize the bridge configuration
    pub fn initialize_bridge(
        ctx: Context<InitializeBridge>,
        min_solver_stake: u64,
        protocol_fee_bps: u16,
    ) -> Result<()> {
        require!(protocol_fee_bps <= 1000, ErrorCode::FeeTooHigh); // Max 10%
        
        let config = &mut ctx.accounts.config;
        config.owner = ctx.accounts.owner.key();
        config.min_solver_stake = min_solver_stake;
        config.protocol_fee_bps = protocol_fee_bps;
        config.total_volume = 0;
        config.total_intents = 0;
        config.active_solvers = 0;
        config.fee_vault = ctx.accounts.fee_vault.key();
        config.bump = ctx.bumps.config;
        
        emit!(BridgeInitialized {
            owner: config.owner,
            min_solver_stake,
            protocol_fee_bps,
        });
        
        Ok(())
    }

    // ========================================================================
    // MODEL REGISTRY INSTRUCTIONS
    // ========================================================================

    /// Register a new AI model
    pub fn register_model(
        ctx: Context<RegisterModel>,
        model_id: u64,
        name: String,
        description: String,
        model_type: u8,
        version: String,
        tee_required: bool,
        attestation_required: bool,
        cost_per_inference: u64,
    ) -> Result<()> {
        require!(name.len() <= 32, ErrorCode::NameTooLong);
        require!(description.len() <= 256, ErrorCode::DescriptionTooLong);
        require!(version.len() <= 16, ErrorCode::VersionTooLong);
        require!(model_type <= 3, ErrorCode::InvalidModelType);
        
        let clock = Clock::get()?;
        let model = &mut ctx.accounts.model;
        
        model.model_id = model_id;
        model.name = name.clone();
        model.description = description;
        model.model_type = model_type;
        model.version = version;
        model.owner = ctx.accounts.owner.key();
        model.tee_required = tee_required;
        model.attestation_required = attestation_required;
        model.cost_per_inference = cost_per_inference;
        model.created_at = clock.unix_timestamp;
        model.updated_at = clock.unix_timestamp;
        model.is_active = true;
        model.bump = ctx.bumps.model;
        
        emit!(ModelRegistered {
            model_id,
            name,
            owner: model.owner,
        });
        
        Ok(())
    }

    /// Update model metadata
    pub fn update_model(
        ctx: Context<UpdateModel>,
        name: Option<String>,
        description: Option<String>,
        version: Option<String>,
        cost_per_inference: Option<u64>,
        is_active: Option<bool>,
    ) -> Result<()> {
        let clock = Clock::get()?;
        let model = &mut ctx.accounts.model;
        
        if let Some(n) = name {
            require!(n.len() <= 32, ErrorCode::NameTooLong);
            model.name = n;
        }
        if let Some(d) = description {
            require!(d.len() <= 256, ErrorCode::DescriptionTooLong);
            model.description = d;
        }
        if let Some(v) = version {
            require!(v.len() <= 16, ErrorCode::VersionTooLong);
            model.version = v;
        }
        if let Some(c) = cost_per_inference {
            model.cost_per_inference = c;
        }
        if let Some(a) = is_active {
            model.is_active = a;
        }
        
        model.updated_at = clock.unix_timestamp;
        
        emit!(ModelUpdated {
            model_id: model.model_id,
        });
        
        Ok(())
    }

    /// Create an inference request
    pub fn create_inference_request(
        ctx: Context<CreateInferenceRequest>,
        request_id: u64,
        prompt_hash: [u8; 32],
        require_attestation: bool,
    ) -> Result<()> {
        let model = &ctx.accounts.model;
        require!(model.is_active, ErrorCode::ModelNotActive);
        
        if model.attestation_required {
            require!(require_attestation, ErrorCode::AttestationRequired);
        }
        
        // Transfer inference cost
        let transfer_ctx = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            anchor_lang::system_program::Transfer {
                from: ctx.accounts.user.to_account_info(),
                to: ctx.accounts.model_owner.to_account_info(),
            },
        );
        anchor_lang::system_program::transfer(transfer_ctx, model.cost_per_inference)?;
        
        let clock = Clock::get()?;
        let request = &mut ctx.accounts.request;
        
        request.request_id = request_id;
        request.model_id = model.model_id;
        request.user = ctx.accounts.user.key();
        request.prompt_hash = prompt_hash;
        request.require_attestation = require_attestation;
        request.created_at = clock.unix_timestamp;
        request.status = 0; // Pending
        request.result_hash = [0u8; 32];
        request.attestation_hash = [0u8; 32];
        request.bump = ctx.bumps.request;
        
        // Update user metrics
        let metrics = &mut ctx.accounts.user_metrics;
        if metrics.user == Pubkey::default() {
            metrics.user = ctx.accounts.user.key();
            metrics.bump = ctx.bumps.user_metrics;
        }
        metrics.total_inferences += 1;
        metrics.total_cost += model.cost_per_inference;
        
        emit!(InferenceRequestCreated {
            request_id,
            model_id: model.model_id,
            user: ctx.accounts.user.key(),
        });
        
        Ok(())
    }

    /// Initialize process inference computation definition
    pub fn init_process_inference_comp_def(ctx: Context<InitProcessInferenceCompDef>) -> Result<()> {
        init_comp_def(ctx.accounts, 0, None, None)?;
        Ok(())
    }

    /// Queue encrypted inference processing
    pub fn process_inference(
        ctx: Context<ProcessInference>,
        computation_offset: u64,
        encrypted_prompt_hash: [u8; 32],
        encrypted_model_id: [u8; 32],
        encrypted_nonce: [u8; 32],
        attestation_key: [u8; 32],
        observer_pub_key: [u8; 32],
        observer_nonce: u128,
    ) -> Result<()> {
        ctx.accounts.sign_pda_account.bump = ctx.bumps.sign_pda_account;
        
        let args = vec![
            Argument::ArcisPubkey(observer_pub_key),
            Argument::PlaintextU128(observer_nonce),
            Argument::EncryptedBytes32(encrypted_prompt_hash),
            Argument::EncryptedBytes32(encrypted_model_id),
            Argument::EncryptedBytes32(encrypted_nonce),
            Argument::PlaintextBytes32(attestation_key),
        ];
        
        queue_computation(
            ctx.accounts,
            computation_offset,
            args,
            None,
            vec![ProcessInferenceCallback::callback_ix(&[])],
            1,
        )?;
        
        Ok(())
    }

    /// Callback for process inference completion
    #[arcium_callback(encrypted_ix = "process_inference")]
    pub fn process_inference_callback(
        ctx: Context<ProcessInferenceCallback>,
        output: ComputationOutputs<ProcessInferenceOutput>,
    ) -> Result<()> {
        let result = match output {
            ComputationOutputs::Success(ProcessInferenceOutput { field_0 }) => field_0,
            _ => return Err(ErrorCode::AbortedComputation.into()),
        };
        
        emit!(InferenceProcessed {
            result_hash: result.ciphertexts[0],
            nonce: result.nonce.to_le_bytes(),
        });
        
        Ok(())
    }

    /// Store inference result (called by TEE service/callback)
    pub fn store_inference_result(
        ctx: Context<StoreInferenceResult>,
        result_hash: [u8; 32],
        attestation_hash: [u8; 32],
        verified: bool,
    ) -> Result<()> {
        let request = &mut ctx.accounts.request;
        
        request.result_hash = result_hash;
        request.attestation_hash = attestation_hash;
        request.status = if verified { 2 } else { 3 }; // Completed or Failed
        
        // Update user metrics
        let metrics = &mut ctx.accounts.user_metrics;
        if verified {
            metrics.successful_inferences += 1;
        } else {
            metrics.failed_inferences += 1;
        }
        
        emit!(InferenceResultStored {
            request_id: request.request_id,
            verified,
        });
        
        Ok(())
    }

    // ========================================================================
    // BATCH INFERENCE INSTRUCTIONS
    // ========================================================================

    /// Create a batch inference request
    pub fn create_batch_inference(
        ctx: Context<CreateBatchInference>,
        batch_id: u64,
        prompt_count: u32,
        require_attestation: bool,
    ) -> Result<()> {
        require!(prompt_count > 0, ErrorCode::EmptyBatch);
        require!(prompt_count <= 100, ErrorCode::BatchTooLarge);
        
        let model = &ctx.accounts.model;
        require!(model.is_active, ErrorCode::ModelNotActive);
        
        // Transfer total inference cost
        let total_cost = model.cost_per_inference.checked_mul(prompt_count as u64)
            .ok_or(ErrorCode::Overflow)?;
        
        let transfer_ctx = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            anchor_lang::system_program::Transfer {
                from: ctx.accounts.user.to_account_info(),
                to: ctx.accounts.model_owner.to_account_info(),
            },
        );
        anchor_lang::system_program::transfer(transfer_ctx, total_cost)?;
        
        let clock = Clock::get()?;
        let batch = &mut ctx.accounts.batch;
        
        batch.batch_id = batch_id;
        batch.model_id = model.model_id;
        batch.prompt_count = prompt_count;
        batch.user = ctx.accounts.user.key();
        batch.require_attestation = require_attestation;
        batch.created_at = clock.unix_timestamp;
        batch.completed_count = 0;
        batch.failed_count = 0;
        batch.bump = ctx.bumps.batch;
        
        // Update user metrics
        let metrics = &mut ctx.accounts.user_metrics;
        if metrics.user == Pubkey::default() {
            metrics.user = ctx.accounts.user.key();
            metrics.bump = ctx.bumps.user_metrics;
        }
        metrics.total_inferences += prompt_count as u64;
        metrics.total_cost += total_cost;
        
        emit!(BatchInferenceCreated {
            batch_id,
            model_id: model.model_id,
            prompt_count,
        });
        
        Ok(())
    }

    // ========================================================================
    // SILENCE BRIDGE INSTRUCTIONS
    // ========================================================================

    /// Register as a solver
    pub fn register_solver(
        ctx: Context<RegisterSolver>,
        supported_chains: u8,
    ) -> Result<()> {
        require!(supported_chains > 0, ErrorCode::NoSupportedChains);
        
        let config = &ctx.accounts.config;
        let stake = ctx.accounts.user.lamports();
        
        // Note: In production, we'd transfer stake to a vault
        // For now, we just verify they have enough
        require!(stake >= config.min_solver_stake, ErrorCode::InsufficientStake);
        
        let clock = Clock::get()?;
        let solver = &mut ctx.accounts.solver;
        
        solver.solver_id = ctx.accounts.user.key();
        solver.supported_chains = supported_chains;
        solver.stake = config.min_solver_stake;
        solver.reputation_score = 100; // Starting score
        solver.total_intents_executed = 0;
        solver.successful_intents = 0;
        solver.failed_intents = 0;
        solver.total_volume = 0;
        solver.is_active = true;
        solver.registered_at = clock.unix_timestamp;
        solver.bump = ctx.bumps.solver;
        
        // Update config
        let config = &mut ctx.accounts.config;
        config.active_solvers += 1;
        
        emit!(SolverRegistered {
            solver_id: solver.solver_id,
            supported_chains,
        });
        
        Ok(())
    }

    /// Create a cross-chain intent
    pub fn create_intent(
        ctx: Context<CreateIntent>,
        intent_id: u64,
        destination_chain: Chain,
        destination_amount_commitment: [u8; 32],
        destination_token_hash: [u8; 32],
        recipient_hash: [u8; 32],
        is_shielded: bool,
        ttl_seconds: i64,
        source_amount: u64,
    ) -> Result<()> {
        require!(source_amount > 0, ErrorCode::ZeroDeposit);
        
        // Transfer funds from creator to intent vault (escrow)
        let transfer_ctx = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            anchor_lang::system_program::Transfer {
                from: ctx.accounts.creator.to_account_info(),
                to: ctx.accounts.intent_vault.to_account_info(),
            },
        );
        anchor_lang::system_program::transfer(transfer_ctx, source_amount)?;
        
        let clock = Clock::get()?;
        let intent = &mut ctx.accounts.intent;
        
        intent.intent_id = intent_id;
        intent.creator = ctx.accounts.creator.key();
        intent.source_chain = Chain::Solana;
        intent.destination_chain = destination_chain.clone();
        intent.source_amount = source_amount;
        intent.destination_amount_commitment = destination_amount_commitment;
        intent.source_token = Pubkey::default(); // Native SOL
        intent.destination_token_hash = destination_token_hash;
        intent.recipient_hash = recipient_hash;
        intent.is_shielded = is_shielded;
        intent.status = IntentStatus::Created;
        intent.solver = None;
        intent.created_at = clock.unix_timestamp;
        intent.expires_at = clock.unix_timestamp + ttl_seconds;
        intent.executed_at = None;
        intent.destination_tx_hash = [0u8; 32];
        intent.privacy_proof = [0u8; 32];
        intent.bump = ctx.bumps.intent;
        
        // Update config stats
        let config = &mut ctx.accounts.config;
        config.total_intents += 1;
        
        emit!(IntentCreated {
            intent_id,
            creator: intent.creator,
            destination_chain,
            source_amount,
            is_shielded,
        });
        
        Ok(())
    }

    /// Match an intent with a solver
    pub fn match_intent(ctx: Context<MatchIntent>) -> Result<()> {
        let intent = &mut ctx.accounts.intent;
        let solver = &ctx.accounts.solver;
        
        require!(intent.status == IntentStatus::Created, ErrorCode::IntentAlreadyMatched);
        require!(solver.is_active, ErrorCode::SolverNotActive);
        
        let clock = Clock::get()?;
        require!(clock.unix_timestamp < intent.expires_at, ErrorCode::IntentExpired);
        
        // Verify solver supports required chains
        let dest_chain_bit = match intent.destination_chain {
            Chain::Solana => 0b001,
            Chain::Near => 0b010,
            Chain::Zcash => 0b100,
        };
        require!(
            (solver.supported_chains & dest_chain_bit) != 0,
            ErrorCode::ChainNotSupported
        );
        
        intent.status = IntentStatus::Matched;
        intent.solver = Some(ctx.accounts.solver_authority.key());
        
        emit!(IntentMatched {
            intent_id: intent.intent_id,
            solver: ctx.accounts.solver_authority.key(),
        });
        
        Ok(())
    }

    /// Execute intent (called by solver after cross-chain transfer)
    pub fn execute_intent(
        ctx: Context<ExecuteIntent>,
        destination_tx_hash: [u8; 32],
        privacy_proof: Option<[u8; 32]>,
    ) -> Result<()> {
        let intent = &mut ctx.accounts.intent;
        
        require!(
            intent.solver == Some(ctx.accounts.solver_authority.key()),
            ErrorCode::NotMatchedSolver
        );
        require!(intent.status == IntentStatus::Matched, ErrorCode::InvalidIntentStatus);
        
        let clock = Clock::get()?;
        
        intent.status = IntentStatus::Executed;
        intent.executed_at = Some(clock.unix_timestamp);
        intent.destination_tx_hash = destination_tx_hash;
        if let Some(proof) = privacy_proof {
            intent.privacy_proof = proof;
        }
        
        emit!(IntentExecuted {
            intent_id: intent.intent_id,
            destination_tx_hash,
        });
        
        Ok(())
    }

    /// Settle intent and distribute rewards
    pub fn settle_intent(ctx: Context<SettleIntent>) -> Result<()> {
        let intent = &mut ctx.accounts.intent;
        let solver = &mut ctx.accounts.solver;
        let config = &mut ctx.accounts.config;
        
        require!(intent.status == IntentStatus::Executed, ErrorCode::IntentNotExecuted);
        
        // Calculate fees
        let protocol_fee = (intent.source_amount as u128)
            .checked_mul(config.protocol_fee_bps as u128)
            .ok_or(ErrorCode::Overflow)?
            .checked_div(10000)
            .ok_or(ErrorCode::Overflow)? as u64;
        let solver_reward = intent.source_amount.checked_sub(protocol_fee)
            .ok_or(ErrorCode::Overflow)?;
        
        // Transfer solver reward (from intent vault to solver)
        **ctx.accounts.intent_vault.try_borrow_mut_lamports()? -= solver_reward;
        **ctx.accounts.solver_authority.try_borrow_mut_lamports()? += solver_reward;
        
        // Transfer protocol fee to fee vault
        **ctx.accounts.intent_vault.try_borrow_mut_lamports()? -= protocol_fee;
        **ctx.accounts.fee_vault.try_borrow_mut_lamports()? += protocol_fee;
        
        // Update solver stats
        solver.total_intents_executed += 1;
        solver.successful_intents += 1;
        solver.total_volume += intent.source_amount;
        solver.reputation_score = solver.reputation_score.saturating_add(1);
        
        // Update intent status
        intent.status = IntentStatus::Settled;
        
        // Update config stats
        config.total_volume += intent.source_amount;
        
        emit!(IntentSettled {
            intent_id: intent.intent_id,
            solver_reward,
            protocol_fee,
        });
        
        Ok(())
    }

    /// Mark intent as failed and refund creator
    pub fn fail_intent(ctx: Context<FailIntent>) -> Result<()> {
        let intent = &mut ctx.accounts.intent;
        let solver = &mut ctx.accounts.solver;
        
        require!(
            intent.solver == Some(ctx.accounts.solver_authority.key()),
            ErrorCode::NotMatchedSolver
        );
        
        // Refund creator
        **ctx.accounts.intent_vault.try_borrow_mut_lamports()? -= intent.source_amount;
        **ctx.accounts.creator.try_borrow_mut_lamports()? += intent.source_amount;
        
        // Update solver stats
        solver.failed_intents += 1;
        solver.reputation_score = solver.reputation_score.saturating_sub(5);
        
        // Update intent status
        intent.status = IntentStatus::Failed;
        
        emit!(IntentFailed {
            intent_id: intent.intent_id,
        });
        
        Ok(())
    }

    // ========================================================================
    // ENCRYPTED COMPUTATION INSTRUCTIONS
    // ========================================================================

    /// Initialize verify intent amounts computation definition
    pub fn init_verify_intent_amounts_comp_def(ctx: Context<InitVerifyIntentAmountsCompDef>) -> Result<()> {
        init_comp_def(ctx.accounts, 0, None, None)?;
        Ok(())
    }

    /// Verify encrypted intent amounts
    pub fn verify_intent_amounts(
        ctx: Context<VerifyIntentAmounts>,
        computation_offset: u64,
        one_time_pub_key: [u8; 32],
        one_time_nonce: u128,
        encrypted_source_amount: [u8; 32],
        encrypted_destination_amount: [u8; 32],
        expected_rate_bps: u64,
        min_source_amount: u128,
        protocol_fee_bps: u64,
        observer_pub_key: [u8; 32],
        observer_nonce: u128,
    ) -> Result<()> {
        ctx.accounts.sign_pda_account.bump = ctx.bumps.sign_pda_account;
        
        let args = vec![
            Argument::ArcisPubkey(one_time_pub_key),
            Argument::PlaintextU128(one_time_nonce),
            Argument::EncryptedU128(encrypted_source_amount),
            Argument::EncryptedU128(encrypted_destination_amount),
            Argument::PlaintextU64(expected_rate_bps),
            Argument::PlaintextU128(min_source_amount),
            Argument::PlaintextU64(protocol_fee_bps),
            Argument::ArcisPubkey(observer_pub_key),
            Argument::PlaintextU128(observer_nonce),
        ];
        
        queue_computation(
            ctx.accounts,
            computation_offset,
            args,
            None,
            vec![VerifyIntentAmountsCallback::callback_ix(&[])],
            1,
        )?;
        
        Ok(())
    }

    /// Callback for verify intent amounts
    #[arcium_callback(encrypted_ix = "verify_intent_amounts")]
    pub fn verify_intent_amounts_callback(
        ctx: Context<VerifyIntentAmountsCallback>,
        output: ComputationOutputs<VerifyIntentAmountsOutput>,
    ) -> Result<()> {
        let result = match output {
            ComputationOutputs::Success(VerifyIntentAmountsOutput { field_0 }) => field_0,
            _ => return Err(ErrorCode::AbortedComputation.into()),
        };
        
        emit!(IntentAmountsVerified {
            result: result.ciphertexts[0],
            nonce: result.nonce.to_le_bytes(),
        });
        
        Ok(())
    }

    /// Initialize compute settlement computation definition
    pub fn init_compute_settlement_comp_def(ctx: Context<InitComputeSettlementCompDef>) -> Result<()> {
        init_comp_def(ctx.accounts, 0, None, None)?;
        Ok(())
    }

    /// Initialize calculate reputation computation definition
    pub fn init_calculate_reputation_comp_def(ctx: Context<InitCalculateReputationCompDef>) -> Result<()> {
        init_comp_def(ctx.accounts, 0, None, None)?;
        Ok(())
    }

    /// Initialize verify attestation computation definition
    pub fn init_verify_attestation_comp_def(ctx: Context<InitVerifyAttestationCompDef>) -> Result<()> {
        init_comp_def(ctx.accounts, 0, None, None)?;
        Ok(())
    }

    /// Initialize generate privacy proof computation definition
    pub fn init_generate_privacy_proof_comp_def(ctx: Context<InitGeneratePrivacyProofCompDef>) -> Result<()> {
        init_comp_def(ctx.accounts, 0, None, None)?;
        Ok(())
    }

    /// Generate privacy proof for shielded transfer
    pub fn generate_privacy_proof(
        ctx: Context<GeneratePrivacyProof>,
        computation_offset: u64,
        one_time_pub_key: [u8; 32],
        one_time_nonce: u128,
        encrypted_amount: [u8; 32],
        encrypted_blinding: [u8; 32],
        encrypted_recipient_hash: [u8; 32],
        max_amount: u128,
        observer_pub_key: [u8; 32],
        observer_nonce: u128,
    ) -> Result<()> {
        ctx.accounts.sign_pda_account.bump = ctx.bumps.sign_pda_account;
        
        let args = vec![
            Argument::ArcisPubkey(one_time_pub_key),
            Argument::PlaintextU128(one_time_nonce),
            Argument::EncryptedU128(encrypted_amount),
            Argument::EncryptedBytes32(encrypted_blinding),
            Argument::EncryptedBytes32(encrypted_recipient_hash),
            Argument::PlaintextU128(max_amount),
            Argument::ArcisPubkey(observer_pub_key),
            Argument::PlaintextU128(observer_nonce),
        ];
        
        queue_computation(
            ctx.accounts,
            computation_offset,
            args,
            None,
            vec![GeneratePrivacyProofCallback::callback_ix(&[])],
            1,
        )?;
        
        Ok(())
    }

    /// Callback for generate privacy proof
    #[arcium_callback(encrypted_ix = "generate_privacy_proof")]
    pub fn generate_privacy_proof_callback(
        ctx: Context<GeneratePrivacyProofCallback>,
        output: ComputationOutputs<GeneratePrivacyProofOutput>,
    ) -> Result<()> {
        let result = match output {
            ComputationOutputs::Success(GeneratePrivacyProofOutput { field_0 }) => field_0,
            _ => return Err(ErrorCode::AbortedComputation.into()),
        };
        
        emit!(PrivacyProofGenerated {
            commitment: result.ciphertexts[0],
            nonce: result.nonce.to_le_bytes(),
        });
        
        Ok(())
    }

    // ========================================================================
    // ADMIN INSTRUCTIONS
    // ========================================================================

    /// Update protocol fee
    pub fn set_protocol_fee(ctx: Context<AdminConfig>, fee_bps: u16) -> Result<()> {
        require!(fee_bps <= 1000, ErrorCode::FeeTooHigh); // Max 10%
        ctx.accounts.config.protocol_fee_bps = fee_bps;
        
        emit!(ProtocolFeeUpdated { fee_bps });
        Ok(())
    }

    /// Deactivate a solver
    pub fn deactivate_solver(ctx: Context<DeactivateSolver>) -> Result<()> {
        ctx.accounts.solver.is_active = false;
        ctx.accounts.config.active_solvers = ctx.accounts.config.active_solvers.saturating_sub(1);
        
        emit!(SolverDeactivated {
            solver_id: ctx.accounts.solver.solver_id,
        });
        Ok(())
    }
}

// ============================================================================
// ACCOUNT CONTEXTS
// ============================================================================

#[derive(Accounts)]
pub struct InitializeBridge<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    
    #[account(
        init,
        payer = owner,
        space = 8 + BridgeConfig::INIT_SPACE,
        seeds = [b"bridge_config"],
        bump
    )]
    pub config: Account<'info, BridgeConfig>,
    
    /// CHECK: Fee vault account
    #[account(mut)]
    pub fee_vault: AccountInfo<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(model_id: u64)]
pub struct RegisterModel<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    
    #[account(
        init,
        payer = owner,
        space = 8 + ModelMetadata::INIT_SPACE,
        seeds = [b"model", model_id.to_le_bytes().as_ref()],
        bump
    )]
    pub model: Account<'info, ModelMetadata>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateModel<'info> {
    #[account(constraint = owner.key() == model.owner @ ErrorCode::Unauthorized)]
    pub owner: Signer<'info>,
    
    #[account(mut)]
    pub model: Account<'info, ModelMetadata>,
}

#[derive(Accounts)]
#[instruction(request_id: u64)]
pub struct CreateInferenceRequest<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(constraint = model.is_active @ ErrorCode::ModelNotActive)]
    pub model: Account<'info, ModelMetadata>,
    
    /// CHECK: Model owner receives payment
    #[account(mut, constraint = model_owner.key() == model.owner @ ErrorCode::InvalidOwner)]
    pub model_owner: AccountInfo<'info>,
    
    #[account(
        init,
        payer = user,
        space = 8 + InferenceRequest::INIT_SPACE,
        seeds = [b"request", request_id.to_le_bytes().as_ref()],
        bump
    )]
    pub request: Account<'info, InferenceRequest>,
    
    #[account(
        init_if_needed,
        payer = user,
        space = 8 + UserMetrics::INIT_SPACE,
        seeds = [b"user_metrics", user.key().as_ref()],
        bump
    )]
    pub user_metrics: Account<'info, UserMetrics>,
    
    pub system_program: Program<'info, System>,
}

#[init_computation_definition_accounts("process_inference", payer)]
#[derive(Accounts)]
pub struct InitProcessInferenceCompDef<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut, address = derive_mxe_pda!())]
    pub mxe_account: Box<Account<'info, MXEAccount>>,
    #[account(mut)]
    /// CHECK: comp_def_account, checked by arcium program
    pub comp_def_account: UncheckedAccount<'info>,
    pub arcium_program: Program<'info, Arcium>,
    pub system_program: Program<'info, System>,
}

#[queue_computation_accounts("process_inference", payer)]
#[derive(Accounts)]
#[instruction(computation_offset: u64)]
pub struct ProcessInference<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        init_if_needed,
        space = 9,
        payer = payer,
        seeds = [&SIGN_PDA_SEED],
        bump,
        address = derive_sign_pda!(),
    )]
    pub sign_pda_account: Account<'info, SignerAccount>,
    #[account(address = derive_mxe_pda!())]
    pub mxe_account: Account<'info, MXEAccount>,
    #[account(mut, address = derive_mempool_pda!())]
    /// CHECK: mempool_account
    pub mempool_account: UncheckedAccount<'info>,
    #[account(mut, address = derive_execpool_pda!())]
    /// CHECK: executing_pool
    pub executing_pool: UncheckedAccount<'info>,
    #[account(mut, address = derive_comp_pda!(computation_offset))]
    /// CHECK: computation_account
    pub computation_account: UncheckedAccount<'info>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_PROCESS_INFERENCE))]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(mut, address = derive_cluster_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    pub cluster_account: Account<'info, Cluster>,
    #[account(mut, address = ARCIUM_FEE_POOL_ACCOUNT_ADDRESS)]
    pub pool_account: Account<'info, FeePool>,
    #[account(address = ARCIUM_CLOCK_ACCOUNT_ADDRESS)]
    pub clock_account: Account<'info, ClockAccount>,
    pub system_program: Program<'info, System>,
    pub arcium_program: Program<'info, Arcium>,
}

#[callback_accounts("process_inference")]
#[derive(Accounts)]
pub struct ProcessInferenceCallback<'info> {
    pub arcium_program: Program<'info, Arcium>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_PROCESS_INFERENCE))]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(address = ::anchor_lang::solana_program::sysvar::instructions::ID)]
    /// CHECK: instructions_sysvar
    pub instructions_sysvar: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct StoreInferenceResult<'info> {
    #[account(constraint = authority.key() == request.user @ ErrorCode::Unauthorized)]
    pub authority: Signer<'info>,
    
    #[account(mut)]
    pub request: Account<'info, InferenceRequest>,
    
    #[account(
        mut,
        seeds = [b"user_metrics", request.user.as_ref()],
        bump = user_metrics.bump
    )]
    pub user_metrics: Account<'info, UserMetrics>,
}

#[derive(Accounts)]
#[instruction(batch_id: u64)]
pub struct CreateBatchInference<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(constraint = model.is_active @ ErrorCode::ModelNotActive)]
    pub model: Account<'info, ModelMetadata>,
    
    /// CHECK: Model owner receives payment
    #[account(mut, constraint = model_owner.key() == model.owner @ ErrorCode::InvalidOwner)]
    pub model_owner: AccountInfo<'info>,
    
    #[account(
        init,
        payer = user,
        space = 8 + BatchInference::INIT_SPACE,
        seeds = [b"batch", batch_id.to_le_bytes().as_ref()],
        bump
    )]
    pub batch: Account<'info, BatchInference>,
    
    #[account(
        init_if_needed,
        payer = user,
        space = 8 + UserMetrics::INIT_SPACE,
        seeds = [b"user_metrics", user.key().as_ref()],
        bump
    )]
    pub user_metrics: Account<'info, UserMetrics>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RegisterSolver<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(mut, seeds = [b"bridge_config"], bump = config.bump)]
    pub config: Account<'info, BridgeConfig>,
    
    #[account(
        init,
        payer = user,
        space = 8 + Solver::INIT_SPACE,
        seeds = [b"solver", user.key().as_ref()],
        bump
    )]
    pub solver: Account<'info, Solver>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(intent_id: u64)]
pub struct CreateIntent<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,
    
    #[account(mut, seeds = [b"bridge_config"], bump = config.bump)]
    pub config: Account<'info, BridgeConfig>,
    
    #[account(
        init,
        payer = creator,
        space = 8 + Intent::INIT_SPACE,
        seeds = [b"intent", intent_id.to_le_bytes().as_ref()],
        bump
    )]
    pub intent: Account<'info, Intent>,
    
    /// CHECK: Intent vault PDA holding escrowed funds
    #[account(
        mut,
        seeds = [b"intent_vault", intent_id.to_le_bytes().as_ref()],
        bump
    )]
    pub intent_vault: AccountInfo<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct MatchIntent<'info> {
    pub solver_authority: Signer<'info>,
    
    #[account(
        mut,
        constraint = intent.status == IntentStatus::Created @ ErrorCode::IntentAlreadyMatched
    )]
    pub intent: Account<'info, Intent>,
    
    #[account(
        seeds = [b"solver", solver_authority.key().as_ref()],
        bump = solver.bump,
        constraint = solver.is_active @ ErrorCode::SolverNotActive
    )]
    pub solver: Account<'info, Solver>,
}

#[derive(Accounts)]
pub struct ExecuteIntent<'info> {
    pub solver_authority: Signer<'info>,
    
    #[account(
        mut,
        constraint = intent.solver == Some(solver_authority.key()) @ ErrorCode::NotMatchedSolver
    )]
    pub intent: Account<'info, Intent>,
}

#[derive(Accounts)]
pub struct SettleIntent<'info> {
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        constraint = intent.status == IntentStatus::Executed @ ErrorCode::IntentNotExecuted
    )]
    pub intent: Account<'info, Intent>,
    
    /// CHECK: Intent vault PDA holding escrowed funds - constrained by seeds
    #[account(
        mut,
        seeds = [b"intent_vault", intent.intent_id.to_le_bytes().as_ref()],
        bump
    )]
    pub intent_vault: AccountInfo<'info>,
    
    #[account(
        mut,
        seeds = [b"solver", intent.solver.unwrap().as_ref()],
        bump = solver.bump
    )]
    pub solver: Account<'info, Solver>,
    
    /// CHECK: Solver authority receives reward
    #[account(mut, constraint = solver_authority.key() == intent.solver.unwrap() @ ErrorCode::NotMatchedSolver)]
    pub solver_authority: AccountInfo<'info>,
    
    #[account(mut, seeds = [b"bridge_config"], bump = config.bump)]
    pub config: Account<'info, BridgeConfig>,
    
    /// CHECK: Fee vault receives protocol fee
    #[account(mut, constraint = fee_vault.key() == config.fee_vault @ ErrorCode::InvalidFeeVault)]
    pub fee_vault: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct FailIntent<'info> {
    pub solver_authority: Signer<'info>,
    
    #[account(
        mut,
        constraint = intent.solver == Some(solver_authority.key()) @ ErrorCode::NotMatchedSolver
    )]
    pub intent: Account<'info, Intent>,
    
    /// CHECK: Intent vault PDA holding escrowed funds - constrained by seeds
    #[account(
        mut,
        seeds = [b"intent_vault", intent.intent_id.to_le_bytes().as_ref()],
        bump
    )]
    pub intent_vault: AccountInfo<'info>,
    
    /// CHECK: Creator receives refund
    #[account(mut, constraint = creator.key() == intent.creator @ ErrorCode::InvalidCreator)]
    pub creator: AccountInfo<'info>,
    
    #[account(
        mut,
        seeds = [b"solver", solver_authority.key().as_ref()],
        bump = solver.bump
    )]
    pub solver: Account<'info, Solver>,
}

// Encrypted computation account contexts
#[init_computation_definition_accounts("verify_intent_amounts", payer)]
#[derive(Accounts)]
pub struct InitVerifyIntentAmountsCompDef<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut, address = derive_mxe_pda!())]
    pub mxe_account: Box<Account<'info, MXEAccount>>,
    #[account(mut)]
    /// CHECK: comp_def_account
    pub comp_def_account: UncheckedAccount<'info>,
    pub arcium_program: Program<'info, Arcium>,
    pub system_program: Program<'info, System>,
}

#[queue_computation_accounts("verify_intent_amounts", payer)]
#[derive(Accounts)]
#[instruction(computation_offset: u64)]
pub struct VerifyIntentAmounts<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        init_if_needed,
        space = 9,
        payer = payer,
        seeds = [&SIGN_PDA_SEED],
        bump,
        address = derive_sign_pda!(),
    )]
    pub sign_pda_account: Account<'info, SignerAccount>,
    #[account(address = derive_mxe_pda!())]
    pub mxe_account: Account<'info, MXEAccount>,
    #[account(mut, address = derive_mempool_pda!())]
    /// CHECK: mempool_account
    pub mempool_account: UncheckedAccount<'info>,
    #[account(mut, address = derive_execpool_pda!())]
    /// CHECK: executing_pool
    pub executing_pool: UncheckedAccount<'info>,
    #[account(mut, address = derive_comp_pda!(computation_offset))]
    /// CHECK: computation_account
    pub computation_account: UncheckedAccount<'info>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_VERIFY_INTENT_AMOUNTS))]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(mut, address = derive_cluster_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    pub cluster_account: Account<'info, Cluster>,
    #[account(mut, address = ARCIUM_FEE_POOL_ACCOUNT_ADDRESS)]
    pub pool_account: Account<'info, FeePool>,
    #[account(address = ARCIUM_CLOCK_ACCOUNT_ADDRESS)]
    pub clock_account: Account<'info, ClockAccount>,
    pub system_program: Program<'info, System>,
    pub arcium_program: Program<'info, Arcium>,
}

#[callback_accounts("verify_intent_amounts")]
#[derive(Accounts)]
pub struct VerifyIntentAmountsCallback<'info> {
    pub arcium_program: Program<'info, Arcium>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_VERIFY_INTENT_AMOUNTS))]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(address = ::anchor_lang::solana_program::sysvar::instructions::ID)]
    /// CHECK: instructions_sysvar
    pub instructions_sysvar: AccountInfo<'info>,
}

#[init_computation_definition_accounts("compute_settlement", payer)]
#[derive(Accounts)]
pub struct InitComputeSettlementCompDef<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut, address = derive_mxe_pda!())]
    pub mxe_account: Box<Account<'info, MXEAccount>>,
    #[account(mut)]
    /// CHECK: comp_def_account
    pub comp_def_account: UncheckedAccount<'info>,
    pub arcium_program: Program<'info, Arcium>,
    pub system_program: Program<'info, System>,
}

#[init_computation_definition_accounts("calculate_reputation", payer)]
#[derive(Accounts)]
pub struct InitCalculateReputationCompDef<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut, address = derive_mxe_pda!())]
    pub mxe_account: Box<Account<'info, MXEAccount>>,
    #[account(mut)]
    /// CHECK: comp_def_account
    pub comp_def_account: UncheckedAccount<'info>,
    pub arcium_program: Program<'info, Arcium>,
    pub system_program: Program<'info, System>,
}

#[init_computation_definition_accounts("verify_attestation", payer)]
#[derive(Accounts)]
pub struct InitVerifyAttestationCompDef<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut, address = derive_mxe_pda!())]
    pub mxe_account: Box<Account<'info, MXEAccount>>,
    #[account(mut)]
    /// CHECK: comp_def_account
    pub comp_def_account: UncheckedAccount<'info>,
    pub arcium_program: Program<'info, Arcium>,
    pub system_program: Program<'info, System>,
}

#[init_computation_definition_accounts("generate_privacy_proof", payer)]
#[derive(Accounts)]
pub struct InitGeneratePrivacyProofCompDef<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut, address = derive_mxe_pda!())]
    pub mxe_account: Box<Account<'info, MXEAccount>>,
    #[account(mut)]
    /// CHECK: comp_def_account
    pub comp_def_account: UncheckedAccount<'info>,
    pub arcium_program: Program<'info, Arcium>,
    pub system_program: Program<'info, System>,
}

#[queue_computation_accounts("generate_privacy_proof", payer)]
#[derive(Accounts)]
#[instruction(computation_offset: u64)]
pub struct GeneratePrivacyProof<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        init_if_needed,
        space = 9,
        payer = payer,
        seeds = [&SIGN_PDA_SEED],
        bump,
        address = derive_sign_pda!(),
    )]
    pub sign_pda_account: Account<'info, SignerAccount>,
    #[account(address = derive_mxe_pda!())]
    pub mxe_account: Account<'info, MXEAccount>,
    #[account(mut, address = derive_mempool_pda!())]
    /// CHECK: mempool_account
    pub mempool_account: UncheckedAccount<'info>,
    #[account(mut, address = derive_execpool_pda!())]
    /// CHECK: executing_pool
    pub executing_pool: UncheckedAccount<'info>,
    #[account(mut, address = derive_comp_pda!(computation_offset))]
    /// CHECK: computation_account
    pub computation_account: UncheckedAccount<'info>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_GENERATE_PRIVACY_PROOF))]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(mut, address = derive_cluster_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    pub cluster_account: Account<'info, Cluster>,
    #[account(mut, address = ARCIUM_FEE_POOL_ACCOUNT_ADDRESS)]
    pub pool_account: Account<'info, FeePool>,
    #[account(address = ARCIUM_CLOCK_ACCOUNT_ADDRESS)]
    pub clock_account: Account<'info, ClockAccount>,
    pub system_program: Program<'info, System>,
    pub arcium_program: Program<'info, Arcium>,
}

#[callback_accounts("generate_privacy_proof")]
#[derive(Accounts)]
pub struct GeneratePrivacyProofCallback<'info> {
    pub arcium_program: Program<'info, Arcium>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_GENERATE_PRIVACY_PROOF))]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(address = ::anchor_lang::solana_program::sysvar::instructions::ID)]
    /// CHECK: instructions_sysvar
    pub instructions_sysvar: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct AdminConfig<'info> {
    #[account(constraint = owner.key() == config.owner @ ErrorCode::Unauthorized)]
    pub owner: Signer<'info>,
    
    #[account(mut, seeds = [b"bridge_config"], bump = config.bump)]
    pub config: Account<'info, BridgeConfig>,
}

#[derive(Accounts)]
pub struct DeactivateSolver<'info> {
    #[account(constraint = owner.key() == config.owner @ ErrorCode::Unauthorized)]
    pub owner: Signer<'info>,
    
    #[account(mut, seeds = [b"bridge_config"], bump = config.bump)]
    pub config: Account<'info, BridgeConfig>,
    
    #[account(mut)]
    pub solver: Account<'info, Solver>,
}

// ============================================================================
// EVENTS
// ============================================================================

#[event]
pub struct BridgeInitialized {
    pub owner: Pubkey,
    pub min_solver_stake: u64,
    pub protocol_fee_bps: u16,
}

#[event]
pub struct ModelRegistered {
    pub model_id: u64,
    pub name: String,
    pub owner: Pubkey,
}

#[event]
pub struct ModelUpdated {
    pub model_id: u64,
}

#[event]
pub struct InferenceRequestCreated {
    pub request_id: u64,
    pub model_id: u64,
    pub user: Pubkey,
}

#[event]
pub struct InferenceProcessed {
    pub result_hash: [u8; 32],
    pub nonce: [u8; 16],
}

#[event]
pub struct InferenceResultStored {
    pub request_id: u64,
    pub verified: bool,
}

#[event]
pub struct BatchInferenceCreated {
    pub batch_id: u64,
    pub model_id: u64,
    pub prompt_count: u32,
}

#[event]
pub struct SolverRegistered {
    pub solver_id: Pubkey,
    pub supported_chains: u8,
}

#[event]
pub struct SolverDeactivated {
    pub solver_id: Pubkey,
}

#[event]
pub struct IntentCreated {
    pub intent_id: u64,
    pub creator: Pubkey,
    pub destination_chain: Chain,
    pub source_amount: u64,
    pub is_shielded: bool,
}

#[event]
pub struct IntentMatched {
    pub intent_id: u64,
    pub solver: Pubkey,
}

#[event]
pub struct IntentExecuted {
    pub intent_id: u64,
    pub destination_tx_hash: [u8; 32],
}

#[event]
pub struct IntentSettled {
    pub intent_id: u64,
    pub solver_reward: u64,
    pub protocol_fee: u64,
}

#[event]
pub struct IntentFailed {
    pub intent_id: u64,
}

#[event]
pub struct IntentAmountsVerified {
    pub result: [u8; 32],
    pub nonce: [u8; 16],
}

#[event]
pub struct PrivacyProofGenerated {
    pub commitment: [u8; 32],
    pub nonce: [u8; 16],
}

#[event]
pub struct ProtocolFeeUpdated {
    pub fee_bps: u16,
}

// ============================================================================
// ERROR CODES
// ============================================================================

#[error_code]
pub enum ErrorCode {
    #[msg("The computation was aborted")]
    AbortedComputation,
    #[msg("Cluster not set")]
    ClusterNotSet,
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Model name too long")]
    NameTooLong,
    #[msg("Model description too long")]
    DescriptionTooLong,
    #[msg("Version string too long")]
    VersionTooLong,
    #[msg("Invalid model type")]
    InvalidModelType,
    #[msg("Model not active")]
    ModelNotActive,
    #[msg("Attestation required for this model")]
    AttestationRequired,
    #[msg("Invalid owner")]
    InvalidOwner,
    #[msg("Empty batch")]
    EmptyBatch,
    #[msg("Batch too large (max 100)")]
    BatchTooLarge,
    #[msg("Arithmetic overflow")]
    Overflow,
    #[msg("Protocol fee too high (max 10%)")]
    FeeTooHigh,
    #[msg("Insufficient solver stake")]
    InsufficientStake,
    #[msg("No supported chains specified")]
    NoSupportedChains,
    #[msg("Zero deposit not allowed")]
    ZeroDeposit,
    #[msg("Intent already matched")]
    IntentAlreadyMatched,
    #[msg("Solver not active")]
    SolverNotActive,
    #[msg("Intent expired")]
    IntentExpired,
    #[msg("Chain not supported by solver")]
    ChainNotSupported,
    #[msg("Not the matched solver")]
    NotMatchedSolver,
    #[msg("Invalid intent status")]
    InvalidIntentStatus,
    #[msg("Intent not executed")]
    IntentNotExecuted,
    #[msg("Invalid fee vault")]
    InvalidFeeVault,
    #[msg("Invalid creator")]
    InvalidCreator,
}
