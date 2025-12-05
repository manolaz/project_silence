use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::collections::{UnorderedMap, LookupMap, Vector};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::json_types::U128;
use near_sdk::{env, near_bindgen, AccountId, Promise, PanicOnDefault, BorshStorageKey, NearToken};

/// Storage keys for collections
#[derive(BorshSerialize, BorshStorageKey)]
#[borsh(crate = "near_sdk::borsh")]
enum StorageKey {
    Intents,
    Solvers,
    Matches,
    ActiveSolvers,
    IntentsByCreator,
    IntentsBySolver,
}

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
    Created,
    Matched,
    Executing,
    Executed,
    Settling,
    Settled,
    Failed,
    Disputed,
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
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
#[borsh(crate = "near_sdk::borsh")]
pub struct SilenceBridgeRegistry {
    pub intents: UnorderedMap<String, Intent>,
    pub solvers: UnorderedMap<AccountId, Solver>,
    pub matches: UnorderedMap<String, IntentMatch>,
    pub active_solvers: Vector<AccountId>,
    pub intents_by_creator: LookupMap<AccountId, Vec<String>>,
    pub intents_by_solver: LookupMap<AccountId, Vec<String>>,
    pub owner: AccountId,
    pub min_solver_stake: u128,
    pub protocol_fee_bps: u32,
    pub total_volume: u128,
}

#[near_bindgen]
impl SilenceBridgeRegistry {
    #[init]
    pub fn new(owner: AccountId, min_solver_stake: U128, protocol_fee_bps: u32) -> Self {
        let min_solver_stake: u128 = min_solver_stake.into();
        Self {
            intents: UnorderedMap::new(StorageKey::Intents),
            solvers: UnorderedMap::new(StorageKey::Solvers),
            matches: UnorderedMap::new(StorageKey::Matches),
            active_solvers: Vector::new(StorageKey::ActiveSolvers),
            intents_by_creator: LookupMap::new(StorageKey::IntentsByCreator),
            intents_by_solver: LookupMap::new(StorageKey::IntentsBySolver),
            owner,
            min_solver_stake,
            protocol_fee_bps,
            total_volume: 0,
        }
    }

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
        assert!(self.intents.get(&intent_id).is_none(), "Intent already exists");
        
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
        
        self.intents.insert(&intent_id, &intent);
        
        let mut creator_intents = self.intents_by_creator.get(&creator).unwrap_or_default();
        creator_intents.push(intent_id.clone());
        self.intents_by_creator.insert(&creator, &creator_intents);
        
        env::log_str(&format!("Intent created: {}", intent_id));
        
        intent
    }

    #[payable]
    pub fn register_solver(&mut self, supported_chains: Vec<Chain>) {
        let solver_id = env::predecessor_account_id();
        let stake = env::attached_deposit().as_yoctonear();
        
        assert!(stake >= self.min_solver_stake, "Insufficient stake");
        assert!(self.solvers.get(&solver_id).is_none(), "Solver already registered");
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
        
        self.solvers.insert(&solver_id, &solver);
        self.active_solvers.push(&solver_id);
        
        env::log_str(&format!("Solver registered: {}", solver_id));
    }

    pub fn match_intent(
        &mut self,
        intent_id: String,
        _proposed_rate: u128,
        _estimated_time: u64,
    ) {
        let solver_id = env::predecessor_account_id();
        
        let mut intent = self.intents.get(&intent_id).expect("Intent not found");
        let solver = self.solvers.get(&solver_id).expect("Solver not found");
        
        assert!(intent.status == IntentStatus::Created, "Intent already matched");
        assert!(solver.is_active, "Solver not active");
        assert!(env::block_timestamp() < intent.expires_at, "Intent expired");
        
        assert!(
            solver.supported_chains.contains(&intent.source_chain)
                && solver.supported_chains.contains(&intent.destination_chain),
            "Solver doesn't support required chains"
        );
        
        intent.status = IntentStatus::Matched;
        intent.solver = Some(solver_id.clone());
        self.intents.insert(&intent_id, &intent);
        
        let mut solver_intents = self.intents_by_solver.get(&solver_id).unwrap_or_default();
        solver_intents.push(intent_id.clone());
        self.intents_by_solver.insert(&solver_id, &solver_intents);
        
        env::log_str(&format!("Intent {} matched with solver {}", intent_id, solver_id));
    }

    pub fn execute_intent(
        &mut self,
        intent_id: String,
        destination_tx_hash: String,
        privacy_proof: Option<String>,
    ) {
        let solver_id = env::predecessor_account_id();
        
        let mut intent = self.intents.get(&intent_id).expect("Intent not found");
        
        assert_eq!(intent.solver, Some(solver_id.clone()), "Not the matched solver");
        assert!(intent.status == IntentStatus::Matched, "Invalid status");
        
        intent.status = IntentStatus::Executed;
        intent.executed_at = Some(env::block_timestamp());
        intent.destination_tx_hash = Some(destination_tx_hash);
        intent.privacy_proof = privacy_proof;
        
        self.intents.insert(&intent_id, &intent);
        
        env::log_str(&format!("Intent {} executed by {}", intent_id, solver_id));
    }

    pub fn settle_intent(&mut self, intent_id: String) {
        let mut intent = self.intents.get(&intent_id).expect("Intent not found");
        
        assert!(intent.status == IntentStatus::Executed, "Not executed");
        
        let solver_id = intent.solver.clone().expect("No solver");
        let mut solver = self.solvers.get(&solver_id).expect("Solver not found");
        
        let protocol_fee = (intent.source_amount * self.protocol_fee_bps as u128) / 10000;
        let solver_reward = intent.source_amount - protocol_fee;
        
        Promise::new(solver_id.clone()).transfer(NearToken::from_yoctonear(solver_reward));
        Promise::new(self.owner.clone()).transfer(NearToken::from_yoctonear(protocol_fee));
        
        solver.total_intents_executed += 1;
        solver.successful_intents += 1;
        solver.total_volume += intent.source_amount;
        solver.reputation_score += 1;
        self.solvers.insert(&solver_id, &solver);
        
        intent.status = IntentStatus::Settled;
        self.intents.insert(&intent_id, &intent);
        
        self.total_volume += intent.source_amount;
        
        env::log_str(&format!("Intent {} settled", intent_id));
    }

    pub fn fail_intent(&mut self, intent_id: String, reason: String) {
        let solver_id = env::predecessor_account_id();
        
        let mut intent = self.intents.get(&intent_id).expect("Intent not found");
        
        assert_eq!(intent.solver, Some(solver_id.clone()), "Not the matched solver");
        
        Promise::new(intent.creator.clone()).transfer(NearToken::from_yoctonear(intent.source_amount));
        
        let mut solver = self.solvers.get(&solver_id).expect("Solver not found");
        solver.failed_intents += 1;
        solver.reputation_score = solver.reputation_score.saturating_sub(5);
        self.solvers.insert(&solver_id, &solver);
        
        intent.status = IntentStatus::Failed;
        self.intents.insert(&intent_id, &intent);
        
        env::log_str(&format!("Intent {} failed: {}", intent_id, reason));
    }

    // View methods

    pub fn get_intent(&self, intent_id: String) -> Option<Intent> {
        self.intents.get(&intent_id)
    }

    pub fn get_intents_by_creator(&self, creator: AccountId) -> Vec<Intent> {
        self.intents_by_creator
            .get(&creator)
            .unwrap_or_default()
            .iter()
            .filter_map(|id| self.intents.get(id))
            .collect()
    }

    pub fn get_solver(&self, solver_id: AccountId) -> Option<Solver> {
        self.solvers.get(&solver_id)
    }

    pub fn get_active_solvers(&self, from_index: u64, limit: u64) -> Vec<Solver> {
        let start = from_index;
        let end = std::cmp::min(from_index + limit, self.active_solvers.len());
        
        (start..end)
            .filter_map(|i| {
                let solver_id = self.active_solvers.get(i)?;
                self.solvers.get(&solver_id)
            })
            .filter(|s| s.is_active)
            .collect()
    }

    pub fn find_solvers_for_chains(
        &self,
        source_chain: Chain,
        destination_chain: Chain,
    ) -> Vec<Solver> {
        (0..self.active_solvers.len())
            .filter_map(|i| {
                let solver_id = self.active_solvers.get(i)?;
                self.solvers.get(&solver_id)
            })
            .filter(|s| {
                s.is_active
                    && s.supported_chains.contains(&source_chain)
                    && s.supported_chains.contains(&destination_chain)
            })
            .collect()
    }

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

    pub fn set_protocol_fee(&mut self, fee_bps: u32) {
        assert_eq!(env::predecessor_account_id(), self.owner, "Only owner");
        assert!(fee_bps <= 1000, "Fee too high");
        self.protocol_fee_bps = fee_bps;
    }

    pub fn deactivate_solver(&mut self, solver_id: AccountId) {
        assert_eq!(env::predecessor_account_id(), self.owner, "Only owner");
        
        let mut solver = self.solvers.get(&solver_id).expect("Solver not found");
        solver.is_active = false;
        self.solvers.insert(&solver_id, &solver);
    }
}
