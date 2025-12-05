use near_sdk::{env, near, collections::{LookupMap, UnorderedSet}, json_types::{Base64VecU8, U128}, AccountId, PanicOnDefault, Promise};

/// Model metadata stored on-chain
#[near(serializers=[borsh, json])]
pub struct ModelMetadata {
    pub model_id: String,
    pub name: String,
    pub description: String,
    pub model_type: String, // "llm", "embedding", "classifier", etc.
    pub version: String,
    pub owner: AccountId,
    pub tee_required: bool,
    pub attestation_required: bool,
    pub cost_per_inference: U128,
    pub created_at: u64,
    pub updated_at: u64,
    pub is_active: bool,
}

/// Inference request stored on-chain
#[near(serializers=[borsh, json])]
pub struct InferenceRequest {
    pub request_id: String,
    pub model_id: String,
    pub user_id: AccountId,
    pub prompt: String,
    pub encrypted_data: Option<Base64VecU8>,
    pub require_attestation: bool,
    pub created_at: u64,
    pub status: String, // "pending", "processing", "completed", "failed"
}

/// Inference result with TEE attestation
#[near(serializers=[borsh, json])]
pub struct InferenceResult {
    pub request_id: String,
    pub result: String,
    pub attestation: Option<TEEAttestation>,
    pub inference_id: String,
    pub timestamp: u64,
    pub verified: bool,
}

/// TEE Attestation proof
#[near(serializers=[borsh, json])]
pub struct TEEAttestation {
    pub enclave_id: String,
    pub attestation_proof: String,
    pub timestamp: u64,
    pub public_key: String,
    pub quote: String,
}

#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct ModelRegistry {
    /// Map of model_id -> ModelMetadata
    models: LookupMap<String, ModelMetadata>,
    /// Set of all registered model IDs
    model_ids: UnorderedSet<String>,
    /// Map of request_id -> InferenceRequest
    requests: LookupMap<String, InferenceRequest>,
    /// Map of request_id -> InferenceResult
    results: LookupMap<String, InferenceResult>,
    /// Owner of the contract
    owner: AccountId,
}

#[near]
impl ModelRegistry {
    #[init]
    pub fn new(owner_id: AccountId) -> Self {
        Self {
            models: LookupMap::new(b"m"),
            model_ids: UnorderedSet::new(b"i"),
            requests: LookupMap::new(b"r"),
            results: LookupMap::new(b"s"),
            owner: owner_id,
        }
    }

    /// Register a new AI model
    pub fn register_model(
        &mut self,
        model_id: String,
        name: String,
        description: String,
        model_type: String,
        version: String,
        tee_required: bool,
        attestation_required: bool,
        cost_per_inference: U128,
    ) {
        self.assert_owner();
        
        let now = env::block_timestamp();
        let model = ModelMetadata {
            model_id: model_id.clone(),
            name,
            description,
            model_type,
            version,
            owner: env::predecessor_account_id(),
            tee_required,
            attestation_required,
            cost_per_inference,
            created_at: now,
            updated_at: now,
            is_active: true,
        };
        
        self.models.insert(&model_id, &model);
        self.model_ids.insert(&model_id);
    }

    /// Update model metadata
    pub fn update_model(
        &mut self,
        model_id: String,
        name: Option<String>,
        description: Option<String>,
        version: Option<String>,
        cost_per_inference: Option<U128>,
        is_active: Option<bool>,
    ) {
        self.assert_owner();
        
        let mut model = self.models.get(&model_id).expect("Model not found");
        
        if let Some(name) = name {
            model.name = name;
        }
        if let Some(description) = description {
            model.description = description;
        }
        if let Some(version) = version {
            model.version = version;
        }
        if let Some(cost) = cost_per_inference {
            model.cost_per_inference = cost;
        }
        if let Some(active) = is_active {
            model.is_active = active;
        }
        
        model.updated_at = env::block_timestamp();
        self.models.insert(&model_id, &model);
    }

    /// Create an inference request
    pub fn create_inference_request(
        &mut self,
        request_id: String,
        model_id: String,
        prompt: String,
        encrypted_data: Option<Base64VecU8>,
        require_attestation: bool,
    ) -> Promise {
        // Verify model exists and is active
        let model = self.models.get(&model_id).expect("Model not found");
        assert!(model.is_active, "Model is not active");
        
        // Verify attestation requirement matches model settings
        if model.attestation_required {
            assert!(require_attestation, "Attestation required for this model");
        }
        
        let request = InferenceRequest {
            request_id: request_id.clone(),
            model_id: model_id.clone(),
            user_id: env::predecessor_account_id(),
            prompt,
            encrypted_data,
            require_attestation,
            created_at: env::block_timestamp(),
            status: "pending".to_string(),
        };
        
        self.requests.insert(&request_id, &request);
        
        // Return promise for async processing (would call TEE service)
        Promise::new(env::current_account_id())
    }

    /// Store inference result (called by TEE service)
    pub fn store_inference_result(
        &mut self,
        request_id: String,
        result: String,
        attestation: Option<TEEAttestation>,
        inference_id: String,
        verified: bool,
    ) {
        self.assert_owner(); // Only owner (TEE service) can store results
        
        let inference_result = InferenceResult {
            request_id: request_id.clone(),
            result,
            attestation,
            inference_id,
            timestamp: env::block_timestamp(),
            verified,
        };
        
        self.results.insert(&request_id, &inference_result);
        
        // Update request status
        if let Some(mut request) = self.requests.get(&request_id) {
            request.status = if verified { "completed" } else { "failed" }.to_string();
            self.requests.insert(&request_id, &request);
        }
    }

    /// Get model metadata
    pub fn get_model(&self, model_id: String) -> Option<ModelMetadata> {
        self.models.get(&model_id)
    }

    /// Get all registered model IDs
    pub fn get_all_models(&self) -> Vec<String> {
        self.model_ids.iter().collect()
    }

    /// Get inference request
    pub fn get_request(&self, request_id: String) -> Option<InferenceRequest> {
        self.requests.get(&request_id)
    }

    /// Get inference result
    pub fn get_result(&self, request_id: String) -> Option<InferenceResult> {
        self.results.get(&request_id)
    }

    /// Verify TEE attestation
    pub fn verify_attestation(&self, attestation: TEEAttestation) -> bool {
        // In production, this would verify against NEAR AI's attestation service
        // For now, basic validation
        !attestation.enclave_id.is_empty() 
            && !attestation.attestation_proof.is_empty()
            && attestation.timestamp > 0
    }

    fn assert_owner(&self) {
        assert_eq!(
            env::predecessor_account_id(),
            self.owner,
            "Only owner can call this method"
        );
    }
}

