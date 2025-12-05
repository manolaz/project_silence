use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::store::{IterableMap, LookupMap, Vector};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near, AccountId, Promise, NearToken, PanicOnDefault};



/// Supported blockchain networks
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, PartialEq)]
#[serde(crate = "near_sdk::serde")]
#[borsh(crate = "near_sdk::borsh")]
pub enum Chain {
    Near,
    Solana,
    Zcash,
}

/// Intent status lifecycle
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, PartialEq)]
#[serde(crate = "near_sdk::serde")]
#[borsh(crate = "near_sdk::borsh")]
pub enum IntentStatus {
    Created,       // Intent created, awaiting solver
    Matched,       // Solver matched, preparing execution
    Executing,     // Crosschain transfer in progress
    Executed,      // Transfer completed on destination
    Settling,      // Settlement in progress
    Settled,       // Fully settled, rewards distributed
    Failed,        // Execution failed
    Disputed,      // Under dispute resolution
}

/// Crosschain transfer intent
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
#[borsh(crate = "near_sdk::borsh")]
pub struct Intent {
    pub intent_id: String,
    pub creator: AccountId,
    pub source_chain: Chain,
    pub destination_chain: Chain,
    pub source_amount: u128,
    pub destination_amount: u128,
    pub source_token: String,
    pub destination_token: String,
    pub recipient: String,
    pub is_shielded: bool,
    pub status: IntentStatus,
    pub solver: Option<AccountId>,
    pub created_at: u64,
    pub expires_at: u64,
    pub executed_at: Option<u64>,
    pub source_tx_hash: Option<String>,
    pub destination_tx_hash: Option<String>,
    pub privacy_proof: Option<String>,
}

/// Solver entity
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
#[borsh(crate = "near_sdk::borsh")]
pub struct Solver {
    pub solver_id: AccountId,
    pub supported_chains: Vec<Chain>,
    pub stake: u128,
    pub reputation_score: u32,
    pub total_intents_executed: u64,
    pub successful_intents: u64,
    pub failed_intents: u64,
    pub total_volume: u128,
    pub is_active: bool,
    pub registered_at: u64,
}

/// Intent match proposal
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
#[borsh(crate = "near_sdk::borsh")]
pub struct IntentMatch {
    pub match_id: String,
    pub intent_id: String,
    pub solver_id: AccountId,
    pub proposed_rate: u128,
    pub estimated_time: u64,
    pub created_at: u64,
}

/// Main contract
#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct SilenceBridgeRegistry {
    /// All intents by intent_id
    pub intents: IterableMap<String, Intent>,
    
    /// Registered solvers
    pub solvers: IterableMap<AccountId, Solver>,
    
    /// Intent matches
    pub matches: IterableMap<String, IntentMatch>,
    
    /// Active solvers list
    pub active_solvers: Vector<AccountId>,
    
    /// Intents by creator
    pub intents_by_creator: LookupMap<AccountId, Vec<String>>,
    
    /// Intents by solver
    pub intents_by_solver: LookupMap<AccountId, Vec<String>>,
    
    /// Contract owner
    pub owner: AccountId,
    
    /// Minimum solver stake
    pub min_solver_stake: u128,
    
    /// Protocol fee (basis points)
    pub protocol_fee_bps: u32,
    
    /// Total volume processed
    pub total_volume: u128,
}



#[near]
impl SilenceBridgeRegistry {
    /// Initialize contract
    #[init]
    pub fn new(owner: AccountId, min_solver_stake: u128, protocol_fee_bps: u32) -> Self {
        Self {
            intents: IterableMap::new(b"i"),
            solvers: IterableMap::new(b"s"),
            matches: IterableMap::new(b"m"),
            active_solvers: Vector::new(b"a"),
            intents_by_creator: LookupMap::new(b"c"),
            intents_by_solver: LookupMap::new(b"v"),
            owner,
            min_solver_stake,
            protocol_fee_bps,
            total_volume: 0,
        }
    }

    /// Create a new crosschain intent
    #[payable]
    pub fn create_intent(
        &mut self,
        intent_id: String,
        destination_chain: Chain,
        destination_amount: u128,
        destination_token: String,
        recipient: String,
        is_shielded: bool,
        ttl_seconds: u64,
    ) -> Intent {
        let creator = env::predecessor_account_id();
        let source_amount = env::attached_deposit().as_yoctonear();
        
        assert!(source_amount > 0, "Must attach deposit");
        assert!(!self.intents.get(&intent_id).is_some(), "Intent already exists");
        
        let intent = Intent {
            intent_id: intent_id.clone(),
            creator: creator.clone(),
            source_chain: Chain::Near,
            destination_chain,
            source_amount,
            destination_amount,
            source_token: "NEAR".to_string(),
            destination_token,
            recipient,
            is_shielded,
            status: IntentStatus::Created,
            solver: None,
            created_at: env::block_timestamp(),
            expires_at: env::block_timestamp() + (ttl_seconds * 1_000_000_000),
            executed_at: None,
            source_tx_hash: None,
            destination_tx_hash: None,
            privacy_proof: None,
        };
        
        self.intents.insert(intent_id.clone(), intent.clone());
        
        // Track by creator
        let mut creator_intents = self.intents_by_creator.get(&creator).cloned().unwrap_or_default();
        creator_intents.push(intent_id.clone());
        self.intents_by_creator.insert(creator.clone(), creator_intents);
        
        env::log_str(&format!("Intent created: {}", intent_id));
        
        intent
    }

    /// Register as a solver
    #[payable]
    pub fn register_solver(&mut self, supported_chains: Vec<Chain>) {
        let solver_id = env::predecessor_account_id();
        let stake = env::attached_deposit().as_yoctonear();
        
        assert!(stake >= self.min_solver_stake, "Insufficient stake");
        assert!(!self.solvers.get(&solver_id).is_some(), "Solver already registered");
        assert!(!supported_chains.is_empty(), "Must support at least one chain");
        
        let solver = Solver {
            solver_id: solver_id.clone(),
            supported_chains,
            stake,
            reputation_score: 100,
            total_intents_executed: 0,
            successful_intents: 0,
            failed_intents: 0,
            total_volume: 0,
            is_active: true,
            registered_at: env::block_timestamp(),
        };
        
        self.solvers.insert(solver_id.clone(), solver);
        self.active_solvers.push(solver_id.clone());
        
        env::log_str(&format!("Solver registered: {}", solver_id));
    }

    /// Match intent with solver
    pub fn match_intent(
        &mut self,
        intent_id: String,
        _proposed_rate: u128,
        _estimated_time: u64,
    ) {
        let solver_id = env::predecessor_account_id();
        
        let mut intent = self.intents.get(&intent_id).expect("Intent not found").clone();
        let solver = self.solvers.get(&solver_id).expect("Solver not found").clone();
        
        assert!(intent.status == IntentStatus::Created, "Intent already matched");
        assert!(solver.is_active, "Solver not active");
        assert!(env::block_timestamp() < intent.expires_at, "Intent expired");
        
        // Verify solver supports required chains
        assert!(
            solver.supported_chains.contains(&intent.source_chain)
                && solver.supported_chains.contains(&intent.destination_chain),
            "Solver doesn't support required chains"
        );
        
        // Update intent
        intent.status = IntentStatus::Matched;
        intent.solver = Some(solver_id.clone());
        self.intents.insert(intent_id.clone(), intent.clone());
        
        // Track by solver
        let mut solver_intents = self.intents_by_solver.get(&solver_id).cloned().unwrap_or_default();
        solver_intents.push(intent_id.clone());
        self.intents_by_solver.insert(solver_id.clone(), solver_intents);
        
        env::log_str(&format!("Intent {} matched with solver {}", intent_id, solver_id));
    }

    /// Execute intent (called by solver after crosschain transfer)
    pub fn execute_intent(
        &mut self,
        intent_id: String,
        destination_tx_hash: String,
        privacy_proof: Option<String>,
    ) {
        let solver_id = env::predecessor_account_id();
        
        let mut intent = self.intents.get(&intent_id).expect("Intent not found").clone();
        
        assert_eq!(intent.solver, Some(solver_id.clone()), "Not the matched solver");
        assert!(intent.status == IntentStatus::Matched, "Invalid status");
        
        intent.status = IntentStatus::Executed;
        intent.executed_at = Some(env::block_timestamp());
        intent.destination_tx_hash = Some(destination_tx_hash);
        intent.privacy_proof = privacy_proof;
        
        self.intents.insert(intent_id.clone(), intent.clone());
        
        env::log_str(&format!("Intent {} executed by {}", intent_id, solver_id));
    }

    /// Settle intent and distribute rewards
    pub fn settle_intent(&mut self, intent_id: String) {
        let mut intent = self.intents.get(&intent_id).expect("Intent not found").clone();
        
        assert!(intent.status == IntentStatus::Executed, "Not executed");
        
        let solver_id = intent.solver.clone().expect("No solver");
        let mut solver = self.solvers.get(&solver_id).expect("Solver not found").clone();
        
        // Calculate fees
        let protocol_fee = (intent.source_amount * self.protocol_fee_bps as u128) / 10000;
        let solver_reward = intent.source_amount - protocol_fee;
        
        // Transfer to solver
        let _ = Promise::new(solver_id.clone()).transfer(NearToken::from_yoctonear(solver_reward));
        
        // Transfer protocol fee to owner
        let _ = Promise::new(self.owner.clone()).transfer(NearToken::from_yoctonear(protocol_fee));
        
        // Update solver stats
        solver.total_intents_executed += 1;
        solver.successful_intents += 1;
        solver.total_volume += intent.source_amount;
        solver.reputation_score += 1; // Simple reputation increase
        self.solvers.insert(solver_id.clone(), solver.clone());
        
        // Update intent
        intent.status = IntentStatus::Settled;
        self.intents.insert(intent_id.clone(), intent.clone());
        
        // Update total volume
        self.total_volume += intent.source_amount;
        
        env::log_str(&format!("Intent {} settled", intent_id));
    }

    /// Mark intent as failed
    pub fn fail_intent(&mut self, intent_id: String, reason: String) {
        let solver_id = env::predecessor_account_id();
        
        let mut intent = self.intents.get(&intent_id).expect("Intent not found").clone();
        
        assert_eq!(intent.solver, Some(solver_id.clone()), "Not the matched solver");
        
        // Refund creator
        let _ = Promise::new(intent.creator.clone()).transfer(NearToken::from_yoctonear(intent.source_amount));
        
        // Update solver stats
        let mut solver = self.solvers.get(&solver_id).expect("Solver not found").clone();
        solver.failed_intents += 1;
        solver.reputation_score = solver.reputation_score.saturating_sub(5);
        self.solvers.insert(solver_id.clone(), solver.clone());
        
        // Update intent
        intent.status = IntentStatus::Failed;
        self.intents.insert(intent_id.clone(), intent.clone());
        
        env::log_str(&format!("Intent {} failed: {}", intent_id, reason));
    }

    // View methods

    /// Get intent by ID
    pub fn get_intent(&self, intent_id: String) -> Option<Intent> {
        self.intents.get(&intent_id).cloned()
    }

    /// Get intents by creator
    pub fn get_intents_by_creator(&self, creator: AccountId) -> Vec<Intent> {
        self.intents_by_creator
            .get(&creator)
            .cloned()
            .unwrap_or_default()
            .iter()
            .filter_map(|id| self.intents.get(id).cloned())
            .collect()
    }

    /// Get solver info
    pub fn get_solver(&self, solver_id: AccountId) -> Option<Solver> {
        self.solvers.get(&solver_id).cloned()
    }

    /// Get active solvers
    pub fn get_active_solvers(&self, from_index: u64, limit: u64) -> Vec<Solver> {
        let start = from_index;
        let end = std::cmp::min(from_index + limit, self.active_solvers.len() as u64);
        
        (start..end)
            .filter_map(|i| {
                let solver_id = self.active_solvers.get(i as u32)?;
                self.solvers.get(solver_id).cloned()
            })
            .filter(|s| s.is_active)
            .collect()
    }

    /// Get solvers supporting specific chains
    pub fn find_solvers_for_chains(
        &self,
        source_chain: Chain,
        destination_chain: Chain,
    ) -> Vec<Solver> {
        self.active_solvers
            .iter()
            .filter_map(|solver_id| self.solvers.get(solver_id).cloned())
            .filter(|s| {
                s.is_active
                    && s.supported_chains.contains(&source_chain)
                    && s.supported_chains.contains(&destination_chain)
            })
            .collect()
    }

    /// Get contract stats
    pub fn get_stats(&self) -> serde_json::Value {
        serde_json::json!({
            "total_intents": self.intents.len(),
            "total_solvers": self.solvers.len(),
            "active_solvers": self.active_solvers.len(),
            "total_volume": self.total_volume.to_string(),
            "protocol_fee_bps": self.protocol_fee_bps,
        })
    }

    // Admin methods

    /// Update protocol fee
    pub fn set_protocol_fee(&mut self, fee_bps: u32) {
        assert_eq!(env::predecessor_account_id(), self.owner, "Only owner");
        assert!(fee_bps <= 1000, "Fee too high"); // Max 10%
        self.protocol_fee_bps = fee_bps;
    }

    /// Deactivate solver
    pub fn deactivate_solver(&mut self, solver_id: AccountId) {
        assert_eq!(env::predecessor_account_id(), self.owner, "Only owner");
        
        let mut solver = self.solvers.get(&solver_id).expect("Solver not found").clone();
        solver.is_active = false;
        self.solvers.insert(solver_id.clone(), solver);
    }
}
