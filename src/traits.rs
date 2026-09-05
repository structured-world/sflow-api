//! SFLOW API traits — public interface contracts for embedders.
//!
//! These traits are MIT/Apache-2.0. Products depend ONLY on these traits.
//! Implementations live in proprietary sflow-engine crates.

use async_trait::async_trait;
use serde_json::Value;
use uuid::Uuid;

use crate::error::SflowError;
use crate::types::*;

// ---------------------------------------------------------------------------
// WorkflowEngine — full engine trait (standalone + embedded)
// ---------------------------------------------------------------------------

/// Full workflow engine interface.
/// Used internally by sflow-server and by embedded sflow-engine.
#[async_trait]
pub trait WorkflowEngine: Send + Sync {
    /// Start a new workflow instance.
    async fn start(
        &self,
        ns: &NamespaceId,
        def_id: &str,
        context: Value,
        owner: &IdentityRef,
        correlation_keys: Vec<CorrelationKey>,
    ) -> Result<ActorAddress, SflowError>;

    /// Send an event to a workflow instance (async — fire and forget).
    async fn send_event(
        &self,
        target: &ActorAddress,
        event: WorkflowEvent,
    ) -> Result<TransitionResult, SflowError>;

    /// Send a signal (broadcast) to all matching workflows in a namespace.
    async fn send_signal(
        &self,
        ns: &NamespaceId,
        signal: &str,
    ) -> Result<Vec<ActorAddress>, SflowError>;

    /// Query workflow state (read-only, no state change).
    async fn query(&self, target: &ActorAddress, query: &str) -> Result<Value, SflowError>;

    /// Update — send event and wait for transition result (sync).
    async fn update(
        &self,
        target: &ActorAddress,
        event: WorkflowEvent,
    ) -> Result<TransitionResult, SflowError>;

    /// Message correlation — route by business key.
    async fn correlate(
        &self,
        ns: &NamespaceId,
        name: &str,
        key: &str,
        data: Value,
    ) -> Result<Option<ActorAddress>, SflowError>;

    /// Wake a sleeping instance (timer-triggered or external).
    async fn wake(&self, target: &ActorAddress) -> Result<(), SflowError>;

    /// Suspend an instance (admin action).
    async fn suspend(&self, target: &ActorAddress) -> Result<(), SflowError>;

    /// Resume a suspended instance.
    async fn resume(&self, target: &ActorAddress) -> Result<(), SflowError>;

    /// Cancel an instance (sets status to Cancelled, cancels timers).
    async fn cancel(&self, target: &ActorAddress) -> Result<(), SflowError>;

    /// Delete a completed/cancelled/failed instance from persistence.
    async fn delete(&self, target: &ActorAddress) -> Result<(), SflowError>;

    /// Migrate an instance to a new definition version.
    async fn migrate(&self, target: &ActorAddress, plan: &MigrationPlan) -> Result<(), SflowError>;

    /// Search workflow instances by attributes.
    async fn search(
        &self,
        ns: &NamespaceId,
        query: SearchQuery,
    ) -> Result<Vec<WorkflowSummary>, SflowError>;

    /// Get full instance state.
    async fn get_state(&self, target: &ActorAddress) -> Result<WorkflowInstance, SflowError>;
}

// ---------------------------------------------------------------------------
// WorkflowBackend — product embedding trait (stable, minimal)
// ---------------------------------------------------------------------------

/// Minimal workflow backend trait for product embedding.
/// Every backend (sflow-engine, Temporal, Camunda) MUST implement this.
/// Stable API surface — semver-guarded.
#[async_trait]
pub trait WorkflowBackend: Send + Sync {
    /// Create and start a new workflow instance.
    async fn create(
        &self,
        ns: &NamespaceId,
        def_id: &str,
        context: Value,
        owner: &IdentityRef,
        correlation_keys: Vec<CorrelationKey>,
    ) -> Result<Uuid, SflowError>;

    /// Send an event to a workflow instance.
    async fn send_event(&self, instance_id: &Uuid, event: WorkflowEvent) -> Result<(), SflowError>;

    /// Query workflow state (read-only).
    async fn query(&self, instance_id: &Uuid) -> Result<WorkflowInstance, SflowError>;

    /// Send a signal (broadcast) to matching workflows.
    async fn signal(&self, ns: &NamespaceId, signal: &str) -> Result<u32, SflowError>;

    /// Cancel a workflow instance.
    async fn cancel(&self, instance_id: &Uuid) -> Result<(), SflowError>;

    /// List workflow instances in a namespace.
    async fn list(
        &self,
        ns: &NamespaceId,
        status: Option<InstanceStatus>,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<WorkflowSummary>, SflowError>;
}

/// Extended workflow backend capabilities (opt-in).
/// SflowEmbeddedBackend implements both WorkflowBackend + WorkflowBackendExt.
/// Alternative backends may implement only partial ext.
#[async_trait]
pub trait WorkflowBackendExt: WorkflowBackend {
    /// Message correlation — route by business key.
    async fn correlate(
        &self,
        ns: &NamespaceId,
        name: &str,
        key: &str,
        data: Value,
    ) -> Result<Option<Uuid>, SflowError>;

    /// Suspend a workflow instance.
    async fn suspend(&self, instance_id: &Uuid) -> Result<(), SflowError>;

    /// Resume a suspended workflow instance.
    async fn resume(&self, instance_id: &Uuid) -> Result<(), SflowError>;

    /// Migrate instance to new definition version.
    async fn migrate(&self, instance_id: &Uuid, plan: &MigrationPlan) -> Result<(), SflowError>;

    /// Search by attributes.
    async fn search(
        &self,
        ns: &NamespaceId,
        query: SearchQuery,
    ) -> Result<Vec<WorkflowSummary>, SflowError>;

    /// Get full instance state including history.
    async fn get_state(&self, instance_id: &Uuid) -> Result<WorkflowInstance, SflowError>;
}

// ---------------------------------------------------------------------------
// PersistenceBackend — pluggable storage
// ---------------------------------------------------------------------------

/// Pluggable persistence backend for workflow state storage.
#[async_trait]
pub trait PersistenceBackend: Send + Sync {
    /// Save (upsert) a workflow instance.
    async fn save_instance(&self, instance: &WorkflowInstance) -> Result<(), SflowError>;

    /// Load a workflow instance by ID.
    async fn load_instance(&self, id: &Uuid) -> Result<Option<WorkflowInstance>, SflowError>;

    /// Delete a workflow instance.
    async fn delete_instance(&self, id: &Uuid) -> Result<(), SflowError>;

    /// Atomically claim an instance for processing (prevents double-processing).
    async fn claim_instance(&self, id: &Uuid, node_id: &str) -> Result<bool, SflowError>;

    /// Append an event record to instance history.
    async fn append_event(&self, id: &Uuid, event: &EventRecord) -> Result<(), SflowError>;

    /// Load event history after a given sequence number.
    async fn load_history(&self, id: &Uuid, after_seq: u64)
        -> Result<Vec<EventRecord>, SflowError>;

    /// Search workflow instances.
    async fn search(
        &self,
        ns: &NamespaceId,
        query: &SearchQuery,
    ) -> Result<Vec<WorkflowSummary>, SflowError>;

    /// Increment usage counter (HMAC-protected, standalone mode).
    async fn increment_usage(&self, counter: &UsageCounter) -> Result<(), SflowError>;

    /// Load usage counter for a period.
    async fn load_usage(&self, period: &UsagePeriod) -> Result<UsageCounter, SflowError>;

    /// Backend capability flags.
    fn capabilities(&self) -> BackendCapabilities;
}

// ---------------------------------------------------------------------------
// ConsensusBackend — pluggable consensus for state transitions
// ---------------------------------------------------------------------------

/// Result of a consensus proposal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProposalResult {
    /// Proposal accepted — state transition committed.
    Accepted,
    /// Proposal rejected — version conflict (stale read).
    Conflict {
        /// Current version in storage.
        current_version: u64,
    },
}

/// Timer claim result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimerClaimResult {
    /// Timer claimed successfully — this node should fire it.
    Claimed,
    /// Timer already claimed by another node.
    AlreadyClaimed {
        /// Node id that holds the claim.
        owner: String,
    },
    /// Timer no longer exists (fired or cancelled).
    NotFound,
}

/// Pluggable consensus backend for coordinating state transitions.
///
/// CE uses `DbConsensus` (optimistic concurrency via persistence backend).
/// EE optionally uses `RaftConsensus` (per-namespace Raft groups, sub-5ms).
#[async_trait]
pub trait ConsensusBackend: Send + Sync {
    /// Propose a state transition for a workflow instance.
    ///
    /// The proposal includes the expected version (optimistic concurrency).
    /// If the current version doesn't match, the proposal is rejected.
    async fn propose(
        &self,
        instance_id: &Uuid,
        expected_version: u64,
        instance: &WorkflowInstance,
        events: &[EventRecord],
    ) -> Result<ProposalResult, SflowError>;

    /// Read the current state of a workflow instance.
    ///
    /// Returns the instance with its current version for use in proposals.
    async fn read(&self, instance_id: &Uuid)
        -> Result<Option<(WorkflowInstance, u64)>, SflowError>;

    /// Claim a timer for firing (distributed coordination).
    ///
    /// Uses SKIP LOCKED (DB) or leader election (Raft) to ensure
    /// only one node fires each timer.
    async fn claim_timer(
        &self,
        timer_id: &str,
        node_id: &str,
    ) -> Result<TimerClaimResult, SflowError>;

    /// Release a claimed timer (e.g., on graceful shutdown).
    async fn release_timer(&self, timer_id: &str, node_id: &str) -> Result<(), SflowError>;

    /// List timers due for firing (before `before` timestamp).
    async fn list_due_timers(
        &self,
        before: chrono::DateTime<chrono::Utc>,
        limit: usize,
    ) -> Result<Vec<DueTimer>, SflowError>;
}

/// A timer that is due for firing.
#[derive(Debug, Clone)]
pub struct DueTimer {
    /// Timer identifier used for claim and release.
    pub timer_id: String,
    /// Instance the timer belongs to.
    pub instance_id: Uuid,
    /// Namespace of the instance.
    pub namespace: String,
    /// Scheduled firing time.
    pub fire_at: chrono::DateTime<chrono::Utc>,
}

// ---------------------------------------------------------------------------
// Backend capabilities
// ---------------------------------------------------------------------------

bitflags::bitflags! {
    /// Capability flags for persistence backends.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct BackendCapabilities: u32 {
        /// Basic CRUD operations on instances.
        const BASIC_CRUD       = 0b0000_0001;
        /// Indexed search queries.
        const SEARCH           = 0b0000_0010;
        /// Real-time change notifications (change streams).
        const CHANGE_STREAMS   = 0b0000_0100;
        /// Safe concurrent access from multiple nodes.
        const MULTI_NODE       = 0b0000_1000;
        /// Cross-region replication with locality.
        const GEO_DISTRIBUTION = 0b0001_0000;
        /// Write-local per region (zone sharding).
        const ZONE_SHARDING    = 0b0010_0000;
        /// Automatic expiration of old data.
        const TTL_INDEXES      = 0b0100_0000;
        /// Multi-document ACID transactions.
        const TRANSACTIONS     = 0b1000_0000;
    }
}

// ---------------------------------------------------------------------------
// EventTransport — pluggable event transport
// ---------------------------------------------------------------------------

/// Pluggable event transport for external events (product ↔ SFLOW).
/// Actor-to-actor messaging uses ActorRouter (internal, via persistence backend).
#[async_trait]
pub trait EventTransport: Send + Sync {
    /// Publish an event to a subject/topic.
    async fn publish(&self, subject: &str, data: &[u8]) -> Result<(), SflowError>;

    /// Subscribe to a subject/topic. Returns a stream of messages.
    async fn subscribe(&self, subject: &str) -> Result<Box<dyn EventStream>, SflowError>;
}

/// Stream of incoming event messages.
#[async_trait]
pub trait EventStream: Send {
    /// Receive the next message. Returns None when stream ends.
    async fn next(&mut self) -> Option<EventMessage>;
}

/// An incoming event message from the transport layer.
#[derive(Debug, Clone)]
pub struct EventMessage {
    /// Subject or topic the message arrived on.
    pub subject: String,
    /// Raw message payload.
    pub data: Vec<u8>,
}

// ---------------------------------------------------------------------------
// ServiceHandler — pluggable service invocation
// ---------------------------------------------------------------------------

/// Pluggable service handler for `invoke` states.
/// Products register services by name (e.g., "chargePayment", "reserveInventory").
/// The engine calls the handler when entering a state with an `invoke` definition.
#[async_trait]
pub trait ServiceHandler: Send + Sync {
    /// Execute a block invocation with typed port inputs and output channel.
    ///
    /// `src` — block name from the invoke definition
    /// `inputs` — named port values (mapped from inputMapping by engine)
    /// `output` — channel for emitting outputs, events, and progress
    ///
    /// Blocks are ISOLATED — they receive only mapped port values,
    /// never workflow context. The engine handles all context read/write
    /// via inputMapping (context → ports) and outputMapping (ports → context).
    ///
    /// Blocks communicate results through the OutputChannel:
    /// - `emit(port, value)` — push value to named output port (streaming OK)
    /// - `event(type, data)` — send event to parent statechart
    /// - `progress(fraction)` — report execution progress
    ///
    /// Returns Ok(BlockResult) with collected outputs on success,
    /// or Err on failure (triggers onError transition if defined).
    async fn invoke(
        &self,
        src: &str,
        inputs: &PortValues,
        output: &dyn OutputChannel,
    ) -> Result<BlockResult, SflowError>;

    /// Get block port metadata for a given source name.
    fn block_metadata(&self, src: &str) -> Option<BlockPortMetadata> {
        let _ = src;
        None
    }
}

// ---------------------------------------------------------------------------
// IdentityProvider — pluggable identity resolution
// ---------------------------------------------------------------------------

/// Pluggable identity provider. SID is one implementation, not a requirement.
#[async_trait]
pub trait IdentityProvider: Send + Sync {
    /// Resolve a token/credential to an identity reference.
    async fn resolve_owner(&self, token: &str) -> Result<IdentityRef, SflowError>;

    /// Check authorization for an action on a resource.
    async fn authorize(
        &self,
        owner: &IdentityRef,
        action: &str,
        resource: &str,
    ) -> Result<bool, SflowError>;
}

// ---------------------------------------------------------------------------
// AuditSink — pluggable audit trail
// ---------------------------------------------------------------------------

/// Pluggable audit sink. SID audit is one implementation.
#[async_trait]
pub trait AuditSink: Send + Sync {
    /// Record an audit entry.
    async fn record(&self, entry: AuditEntry) -> Result<(), SflowError>;
}

// ---------------------------------------------------------------------------
// AiProvider — pluggable LLM provider for AI workflow assistance
// ---------------------------------------------------------------------------

/// Information about an AI model.
#[derive(Debug, Clone)]
pub struct AiModelInfo {
    /// Provider name (e.g. "openai", "anthropic").
    pub provider: String,
    /// Provider-specific model identifier.
    pub model_id: String,
    /// Maximum tokens the model accepts per request.
    pub max_tokens: usize,
    /// Whether the model can be forced to emit JSON.
    pub supports_json_mode: bool,
}

/// A message in a chat conversation.
#[derive(Debug, Clone)]
pub struct AiMessage {
    /// Who authored the message.
    pub role: AiRole,
    /// Message text.
    pub content: String,
}

/// Role in an AI conversation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiRole {
    /// System prompt.
    System,
    /// Human turn.
    User,
    /// Model turn.
    Assistant,
}

/// Pluggable AI provider for LLM-assisted workflow creation.
///
/// CE: user provides API key (bring-your-own).
/// EE: managed endpoint via config.
#[async_trait]
pub trait AiProvider: Send + Sync {
    /// Complete a chat conversation.
    async fn complete(&self, messages: &[AiMessage], json_mode: bool)
        -> Result<String, SflowError>;

    /// Get model information.
    fn model_info(&self) -> AiModelInfo;
}
