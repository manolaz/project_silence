use near_sdk::{env, near, collections::LookupMap, json_types::U128, AccountId, PanicOnDefault, Promise};

/// Batch inference request
#[near(serializers=[borsh, json])]
pub struct BatchInferenceRequest {
    pub batch_id: String,
    pub model_id: String,
    pub prompts: Vec<String>,
    pub user_id: AccountId,
    pub require_attestation: bool,
    pub created_at: u64,
}

/// Streaming inference configuration
#[near(serializers=[borsh, json])]
pub struct StreamingConfig {
    pub stream_id: String,
    pub model_id: String,
    pub prompt: String,
    pub chunk_size: u32,
    pub user_id: AccountId,
    pub created_at: u64,
}

/// Inference metrics
#[near(serializers=[borsh, json])]
pub struct InferenceMetrics {
    pub total_inferences: u64,
    pub successful_inferences: u64,
    pub failed_inferences: u64,
    pub total_cost: U128,
    pub average_latency_ms: u64,
}

#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct InferenceService {
    /// Map of batch_id -> BatchInferenceRequest
    batches: LookupMap<String, BatchInferenceRequest>,
    /// Map of stream_id -> StreamingConfig
    streams: LookupMap<String, StreamingConfig>,
    /// Map of user_id -> InferenceMetrics
    user_metrics: LookupMap<AccountId, InferenceMetrics>,
    /// Owner of the contract
    owner: AccountId,
    /// Model registry contract address
    model_registry: AccountId,
}

#[near]
impl InferenceService {
    #[init]
    pub fn new(owner_id: AccountId, model_registry_id: AccountId) -> Self {
        Self {
            batches: LookupMap::new(b"b"),
            streams: LookupMap::new(b"s"),
            user_metrics: LookupMap::new(b"m"),
            owner: owner_id,
            model_registry: model_registry_id,
        }
    }

    /// Create a batch inference request
    pub fn create_batch_inference(
        &mut self,
        batch_id: String,
        model_id: String,
        prompts: Vec<String>,
        require_attestation: bool,
    ) -> Promise {
        assert!(!prompts.is_empty(), "Prompts cannot be empty");
        assert!(prompts.len() <= 100, "Maximum 100 prompts per batch");
        
        let batch = BatchInferenceRequest {
            batch_id: batch_id.clone(),
            model_id: model_id.clone(),
            prompts: prompts.clone(),
            user_id: env::predecessor_account_id(),
            require_attestation,
            created_at: env::block_timestamp(),
        };
        
        self.batches.insert(&batch_id, &batch);
        
        // Update metrics
        self.update_metrics(&env::predecessor_account_id(), true);
        
        // Return promise to process batch (would call TEE service)
        Promise::new(self.model_registry.clone())
    }

    /// Create a streaming inference request
    pub fn create_streaming_inference(
        &mut self,
        stream_id: String,
        model_id: String,
        prompt: String,
        chunk_size: u32,
        _require_attestation: bool,
    ) {
        assert!(chunk_size > 0 && chunk_size <= 1000, "Invalid chunk size");
        
        let stream = StreamingConfig {
            stream_id: stream_id.clone(),
            model_id,
            prompt,
            chunk_size,
            user_id: env::predecessor_account_id(),
            created_at: env::block_timestamp(),
        };
        
        self.streams.insert(&stream_id, &stream);
    }

    /// Get batch inference request
    pub fn get_batch(&self, batch_id: String) -> Option<BatchInferenceRequest> {
        self.batches.get(&batch_id)
    }

    /// Get streaming config
    pub fn get_stream(&self, stream_id: String) -> Option<StreamingConfig> {
        self.streams.get(&stream_id)
    }

    /// Get user inference metrics
    pub fn get_user_metrics(&self, user_id: AccountId) -> Option<InferenceMetrics> {
        self.user_metrics.get(&user_id)
    }

    /// Update inference metrics
    fn update_metrics(&mut self, user_id: &AccountId, success: bool) {
        let mut metrics = self.user_metrics
            .get(user_id)
            .unwrap_or(InferenceMetrics {
                total_inferences: 0,
                successful_inferences: 0,
                failed_inferences: 0,
                total_cost: U128(0),
                average_latency_ms: 0,
            });
        
        metrics.total_inferences += 1;
        if success {
            metrics.successful_inferences += 1;
        } else {
            metrics.failed_inferences += 1;
        }
        
        self.user_metrics.insert(user_id, &metrics);
    }

    fn assert_owner(&self) {
        assert_eq!(
            env::predecessor_account_id(),
            self.owner,
            "Only owner can call this method"
        );
    }
}

