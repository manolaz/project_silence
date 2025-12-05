use borsh::{BorshSerialize, BorshDeserialize};
use near_sdk::collections::{LookupMap, UnorderedMap};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near, near_bindgen, AccountId, Promise, PanicOnDefault, NearToken};

/// Philanthropic cause registered on NEAR
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct PhilanthropicCause {
    pub cause_id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub recipient_address: AccountId,
    pub verified: bool,
    pub impact_score: u8,
    pub total_donations: u128,
    pub donor_count: u64,
    pub tags: Vec<String>,
    pub created_at: u64,
    pub updated_at: u64,
}

/// Donation record
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Donation {
    pub donation_id: String,
    pub cause_id: String,
    pub donor: AccountId,
    pub amount: u128,
    pub is_crosschain: bool,
    pub source_chain: Option<String>,
    pub bridge_tx_hash: Option<String>,
    pub timestamp: u64,
    pub is_private: bool,
}

/// Crosschain bridge request
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct BridgeRequest {
    pub request_id: String,
    pub from_chain: String,
    pub to_chain: String,
    pub amount: u128,
    pub token: String,
    pub recipient: String,
    pub is_shielded: bool,
    pub status: BridgeStatus,
    pub lock_tx_hash: Option<String>,
    pub mint_tx_hash: Option<String>,
    pub proof_hash: Option<String>,
    pub created_at: u64,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub enum BridgeStatus {
    Pending,
    Locked,
    Proved,
    Minted,
    Completed,
    Failed,
}

/// Main contract structure
#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct PhilanthropyAgent {
    /// Registered causes by cause_id
    pub causes: UnorderedMap<String, PhilanthropicCause>,
    
    /// Donations by donation_id
    pub donations: UnorderedMap<String, Donation>,
    
    /// Bridge requests by request_id
    pub bridge_requests: UnorderedMap<String, BridgeRequest>,
    
    /// Donations by donor account
    pub donations_by_donor: LookupMap<AccountId, Vec<String>>,
    
    /// Donations by cause
    pub donations_by_cause: LookupMap<String, Vec<String>>,
    
    /// Total donations in the system
    pub total_donations: u128,
    
    /// Contract owner
    pub owner: AccountId,
    
    /// Verifiers who can verify causes
    pub verifiers: Vec<AccountId>,
}

#[near_bindgen]
impl PhilanthropyAgent {
    /// Initialize the contract
    #[init]
    pub fn new(owner: AccountId) -> Self {
        Self {
            causes: UnorderedMap::new(b"c"),
            donations: UnorderedMap::new(b"d"),
            bridge_requests: UnorderedMap::new(b"b"),
            donations_by_donor: LookupMap::new(b"dd".as_slice()),
            donations_by_cause: LookupMap::new(b"dc".as_slice()),
            total_donations: 0,
            owner,
            verifiers: Vec::new(),
        }
    }

    /// Register a new philanthropic cause
    pub fn register_cause(
        &mut self,
        cause_id: String,
        name: String,
        description: String,
        category: String,
        recipient_address: AccountId,
        tags: Vec<String>,
    ) -> PhilanthropicCause {
        assert!(
            !self.causes.get(&cause_id).is_some(),
            "Cause already exists"
        );

        let cause = PhilanthropicCause {
            cause_id: cause_id.clone(),
            name,
            description,
            category,
            recipient_address,
            verified: false,
            impact_score: 0,
            total_donations: 0,
            donor_count: 0,
            tags,
            created_at: env::block_timestamp(),
            updated_at: env::block_timestamp(),
        };

        self.causes.insert(&cause_id, &cause);
        
        env::log_str(&format!("Cause registered: {}", cause_id));
        
        cause
    }

    /// Verify a cause (only by verifiers or owner)
    pub fn verify_cause(&mut self, cause_id: String, impact_score: u8) {
        let caller = env::predecessor_account_id();
        assert!(
            caller == self.owner || self.verifiers.contains(&caller),
            "Only verifiers can verify causes"
        );

        let mut cause = self
            .causes
            .get(&cause_id)
            .expect("Cause not found");

        cause.verified = true;
        cause.impact_score = impact_score;
        cause.updated_at = env::block_timestamp();

        self.causes.insert(&cause_id, &cause);
        
        env::log_str(&format!("Cause verified: {}", cause_id));
    }

    /// Make a donation to a cause
    #[payable]
    pub fn donate(
        &mut self,
        cause_id: String,
        is_private: bool,
    ) -> Donation {
        let donor = env::predecessor_account_id();
        let amount = env::attached_deposit();
        let amount_yocto = amount.as_yoctonear();

        assert!(amount_yocto > 0, "Donation amount must be greater than 0");

        let mut cause = self
            .causes
            .get(&cause_id)
            .expect("Cause not found");

        // Transfer to cause recipient
        Promise::new(cause.recipient_address.clone()).transfer(amount);

        // Create donation record
        let donation_id = format!(
            "{}_{}",
            cause_id,
            env::block_timestamp()
        );

        let donation = Donation {
            donation_id: donation_id.clone(),
            cause_id: cause_id.clone(),
            donor: donor.clone(),
            amount: amount_yocto,
            is_crosschain: false,
            source_chain: None,
            bridge_tx_hash: None,
            timestamp: env::block_timestamp(),
            is_private,
        };

        // Update cause stats
        cause.total_donations += amount_yocto;
        cause.donor_count += 1;
        cause.updated_at = env::block_timestamp();
        self.causes.insert(&cause_id, &cause);

        // Store donation
        self.donations.insert(&donation_id, &donation);

        // Update total
        self.total_donations += amount_yocto;

        // Update mappings
        let mut donor_donations = self
            .donations_by_donor
            .get(&donor)
            .unwrap_or_else(Vec::new);
        donor_donations.push(donation_id.clone());
        self.donations_by_donor.insert(&donor, &donor_donations);

        let mut cause_donations = self
            .donations_by_cause
            .get(&cause_id)
            .unwrap_or_else(Vec::new);
        cause_donations.push(donation_id.clone());
        self.donations_by_cause.insert(&cause_id, &cause_donations);

        env::log_str(&format!(
            "Donation: {} NEAR to {} by {}",
            amount,
            cause_id,
            if is_private { "anonymous" } else { donor.as_str() }
        ));

        donation
    }

    /// Create a crosschain bridge request
    pub fn create_bridge_request(
        &mut self,
        request_id: String,
        from_chain: String,
        to_chain: String,
        amount: u128,
        token: String,
        recipient: String,
        is_shielded: bool,
    ) -> BridgeRequest {
        assert!(
            !self.bridge_requests.get(&request_id).is_some(),
            "Bridge request already exists"
        );

        let request = BridgeRequest {
            request_id: request_id.clone(),
            from_chain,
            to_chain,
            amount,
            token,
            recipient,
            is_shielded,
            status: BridgeStatus::Pending,
            lock_tx_hash: None,
            mint_tx_hash: None,
            proof_hash: None,
            created_at: env::block_timestamp(),
        };

        self.bridge_requests.insert(&request_id, &request);
        
        env::log_str(&format!("Bridge request created: {}", request_id));
        
        request
    }

    /// Update bridge request status
    pub fn update_bridge_status(
        &mut self,
        request_id: String,
        status: BridgeStatus,
        lock_tx_hash: Option<String>,
        mint_tx_hash: Option<String>,
        proof_hash: Option<String>,
    ) {
        let mut request = self
            .bridge_requests
            .get(&request_id)
            .expect("Bridge request not found");

        request.status = status;
        if lock_tx_hash.is_some() {
            request.lock_tx_hash = lock_tx_hash;
        }
        if mint_tx_hash.is_some() {
            request.mint_tx_hash = mint_tx_hash;
        }
        if proof_hash.is_some() {
            request.proof_hash = proof_hash;
        }

        self.bridge_requests.insert(&request_id, &request);
        
        env::log_str(&format!("Bridge request updated: {}", request_id));
    }

    // View methods

    /// Get a cause by ID
    pub fn get_cause(&self, cause_id: String) -> Option<PhilanthropicCause> {
        self.causes.get(&cause_id)
    }

    /// Get all causes
    pub fn get_all_causes(&self, from_index: u64, limit: u64) -> Vec<PhilanthropicCause> {
        let keys = self.causes.keys_as_vector();
        let start = from_index;
        let end = std::cmp::min(from_index + limit, keys.len());

        (start..end)
            .filter_map(|index| {
                let key = keys.get(index)?;
                self.causes.get(&key)
            })
            .collect()
    }

    /// Get verified causes
    pub fn get_verified_causes(&self, from_index: u64, limit: u64) -> Vec<PhilanthropicCause> {
        self.get_all_causes(from_index, limit * 2)
            .into_iter()
            .filter(|cause| cause.verified)
            .take(limit as usize)
            .collect()
    }

    /// Get donation by ID
    pub fn get_donation(&self, donation_id: String) -> Option<Donation> {
        self.donations.get(&donation_id)
    }

    /// Get donations by donor
    pub fn get_donations_by_donor(&self, donor: AccountId) -> Vec<Donation> {
        self.donations_by_donor
            .get(&donor)
            .unwrap_or_else(Vec::new)
            .iter()
            .filter_map(|id| self.donations.get(id))
            .collect()
    }

    /// Get donations by cause
    pub fn get_donations_by_cause(&self, cause_id: String) -> Vec<Donation> {
        self.donations_by_cause
            .get(&cause_id)
            .unwrap_or_else(Vec::new)
            .iter()
            .filter_map(|id| self.donations.get(id))
            .collect()
    }

    /// Get bridge request
    pub fn get_bridge_request(&self, request_id: String) -> Option<BridgeRequest> {
        self.bridge_requests.get(&request_id)
    }

    /// Get contract stats
    pub fn get_stats(&self) -> serde_json::Value {
        serde_json::json!({
            "total_donations": self.total_donations.to_string(),
            "total_causes": self.causes.len(),
            "total_bridge_requests": self.bridge_requests.len(),
        })
    }

    // Admin methods

    /// Add a verifier
    pub fn add_verifier(&mut self, verifier: AccountId) {
        assert_eq!(
            env::predecessor_account_id(),
            self.owner,
            "Only owner can add verifiers"
        );

        if !self.verifiers.contains(&verifier) {
            self.verifiers.push(verifier);
        }
    }

    /// Remove a verifier
    pub fn remove_verifier(&mut self, verifier: AccountId) {
        assert_eq!(
            env::predecessor_account_id(),
            self.owner,
            "Only owner can remove verifiers"
        );

        self.verifiers.retain(|v| v != &verifier);
    }
}
