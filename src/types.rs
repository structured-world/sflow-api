//! SFLOW API types — public data types for embedders.
//!
//! These types are the MIT/Apache-2.0 public interface. They contain
//! NO implementation logic — only data structures and enums.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Identity types
// ---------------------------------------------------------------------------

/// Namespace identifier for multi-tenancy isolation.
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct NamespaceId(pub String);

impl NamespaceId {
    /// The `default` namespace used when a caller does not specify one.
    pub fn default_namespace() -> Self {
        Self("default".to_string())
    }
}

/// Reference to an identity (user, service account, system).
/// Opaque to SFLOW — products resolve via IdentityProvider trait.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityRef {
    /// Identity type: "user", "service_account", "system"
    pub kind: String,
    /// Identity value: user sub, service account name, etc.
    pub value: String,
}

// ---------------------------------------------------------------------------
// Workflow mode & status
// ---------------------------------------------------------------------------

/// Workflow execution mode — determines lifecycle semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowMode {
    /// Has final states, completes when reached.
    Process,
    /// No final states, lives indefinitely (entity store pattern).
    Entity,
    /// Always-active stream consumer, forks children on trigger (never sleeps).
    Stream,
}

/// Instance lifecycle status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstanceStatus {
    /// In memory, processing events.
    Active,
    /// Persisted to DB, waiting for timer or event.
    Sleeping,
    /// Final state reached (success).
    Completed,
    /// Final state reached (failure).
    Failed,
    /// Running compensation logic.
    Compensating,
    /// Manually paused by admin.
    Suspended,
    /// Being migrated to new version.
    Migrating,
    /// Cancelled by admin or API request.
    Cancelled,
}

// ---------------------------------------------------------------------------
// Workflow event
// ---------------------------------------------------------------------------

/// An event sent to a workflow instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowEvent {
    /// Event type name (e.g., "INITIATE", "PAYMENT_COMPLETED").
    #[serde(rename = "type")]
    pub event_type: String,
    /// Event payload data.
    #[serde(default)]
    pub data: serde_json::Value,
}

/// Result of processing a transition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionResult {
    /// Whether a transition actually occurred.
    pub transitioned: bool,
    /// Current state(s) after processing.
    pub current_state: StateConfiguration,
    /// Updated context.
    pub context: serde_json::Value,
    /// Events produced by actions during transition.
    pub produced_events: Vec<WorkflowEvent>,
}

/// Payload published to event transport on state changes.
///
/// Sent to NATS subjects `sflow.instance.{ns}.{id}.state` for real-time
/// widget updates (dashboard counters, timelines, live state streaming).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateChangePayload {
    /// Instance whose state changed.
    pub instance_id: uuid::Uuid,
    /// Namespace of the instance.
    pub namespace: String,
    /// Definition the instance runs.
    pub definition_id: String,
    /// Active states before the transition.
    pub from_states: Vec<StateId>,
    /// Active states after the transition.
    pub to_states: Vec<StateId>,
    /// Event type that triggered the transition, if any.
    pub trigger_event: Option<String>,
    /// Context after the transition.
    pub context: serde_json::Value,
    /// Lifecycle status after the transition.
    pub status: InstanceStatus,
    /// When the transition was committed.
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

// ---------------------------------------------------------------------------
// State configuration
// ---------------------------------------------------------------------------

/// Current state configuration — supports parallel states (multiple active).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateConfiguration {
    /// Active state IDs (single for atomic, multiple for parallel regions).
    pub active_states: Vec<StateId>,
}

/// State identifier within a statechart.
pub type StateId = String;

// ---------------------------------------------------------------------------
// Correlation
// ---------------------------------------------------------------------------

/// Correlation key for message routing by business identifier.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationKey {
    /// Correlation name (e.g., "order_id", "customer_id").
    pub name: String,
    /// Correlation value (e.g., "order-789").
    pub value: String,
}

// ---------------------------------------------------------------------------
// Search attributes
// ---------------------------------------------------------------------------

/// Search attribute type declaration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchAttrType {
    /// Exact-match string.
    Keyword,
    /// Full-text searchable string.
    Text,
    /// Signed 64-bit integer.
    Int,
    /// 64-bit float.
    Double,
    /// UTC timestamp.
    Datetime,
    /// Boolean flag.
    Bool,
}

/// Search attribute value (typed).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SearchAttrValue {
    /// Exact-match string.
    Keyword(String),
    /// Full-text searchable string.
    Text(String),
    /// Signed 64-bit integer.
    Int(i64),
    /// 64-bit float.
    Double(f64),
    /// UTC timestamp.
    Datetime(DateTime<Utc>),
    /// Boolean flag.
    Bool(bool),
}

/// Search query for workflow instances.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    /// Filter by instance status.
    pub status: Option<InstanceStatus>,
    /// Filter by definition ID.
    pub definition_id: Option<String>,
    /// Filter by search attributes (key = attr name).
    pub attributes: HashMap<String, SearchAttrValue>,
    /// Maximum results to return.
    pub limit: Option<u32>,
    /// Offset for pagination.
    pub offset: Option<u32>,
}

/// Summary of a workflow instance (for search results).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowSummary {
    /// Instance identifier.
    pub instance_id: Uuid,
    /// Namespace the instance lives in.
    pub namespace: NamespaceId,
    /// Definition the instance runs.
    pub definition_id: String,
    /// Definition version the instance runs.
    pub definition_version: String,
    /// Lifecycle status.
    pub status: InstanceStatus,
    /// Active states.
    pub current_state: StateConfiguration,
    /// Indexed search attributes.
    pub search_attributes: HashMap<String, SearchAttrValue>,
    /// Creation time.
    pub created_at: DateTime<Utc>,
    /// Last state change time.
    pub updated_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Workflow instance
// ---------------------------------------------------------------------------

/// A running workflow instance — full state snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowInstance {
    /// Instance identifier.
    pub instance_id: Uuid,
    /// Optimistic concurrency version (incremented on each state change).
    #[serde(default)]
    pub version: u64,
    /// Namespace the instance lives in.
    pub namespace: NamespaceId,
    /// Definition the instance runs.
    pub definition_id: String,
    /// Definition version the instance runs.
    pub definition_version: String,
    /// Active states.
    pub current_state: StateConfiguration,
    /// Workflow context (extended state).
    pub context: serde_json::Value,
    /// Event history in sequence order.
    pub history: Vec<EventRecord>,
    /// Indexed search attributes.
    pub search_attributes: HashMap<String, SearchAttrValue>,
    /// Correlation keys for message routing.
    pub correlation_keys: Vec<CorrelationKey>,
    /// Creation time.
    pub created_at: DateTime<Utc>,
    /// Last state change time.
    pub updated_at: DateTime<Utc>,
    /// Identity that started the instance.
    pub owner: IdentityRef,
    /// Lifecycle status.
    pub status: InstanceStatus,
    /// Execution mode of the definition.
    pub mode: WorkflowMode,
    /// History of previously active states per compound state path.
    /// Used by history pseudo-states (shallow/deep) for state restoration.
    #[serde(default)]
    pub state_history: HashMap<String, Vec<String>>,
    /// When the instance should be woken (for timer-based transitions).
    pub wake_at: Option<DateTime<Utc>>,
    /// Parent actor address (if spawned as child).
    pub parent: Option<ActorAddress>,
    /// Chain to previous instance (for continue-as-new).
    pub continue_chain: Option<Uuid>,
    /// Child instance IDs (if this is a parent with nested workflows).
    #[serde(default)]
    pub children: Vec<Uuid>,
}

// ---------------------------------------------------------------------------
// Event history
// ---------------------------------------------------------------------------

/// An immutable event record in the instance history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventRecord {
    /// Sequence number (monotonically increasing per instance).
    pub seq: u64,
    /// Record type.
    #[serde(rename = "type")]
    pub record_type: EventRecordType,
    /// Timestamp of the event.
    pub timestamp: DateTime<Utc>,
    /// Additional data (varies by record type).
    #[serde(default)]
    pub data: serde_json::Value,
}

/// Types of event history records.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventRecordType {
    /// Instance created.
    Created,
    /// State transition occurred.
    Transition,
    /// Activity (service invocation) started.
    ActivityStarted,
    /// Activity completed successfully.
    ActivityCompleted,
    /// Activity failed.
    ActivityFailed,
    /// Activity heartbeat received.
    Heartbeat,
    /// Timer fired.
    TimerFired,
    /// Signal received.
    SignalReceived,
    /// Context updated (without state change).
    ContextUpdated,
    /// Instance suspended by admin.
    Suspended,
    /// Instance resumed by admin.
    Resumed,
    /// Migration applied.
    Migrated,
    /// Instance continued as new (chained to successor).
    ContinuedAsNew,
    /// Task dispatched to worker queue.
    TaskDispatched,
    /// Task completed by worker.
    TaskCompleted,
    /// Task failed (may retry).
    TaskFailed,
    /// Task progress update from worker.
    ProgressUpdated,
    /// Compensation step started.
    CompensationStarted,
    /// Compensation step completed.
    CompensationCompleted,
    /// Compensation step failed.
    CompensationFailed,
    /// Child workflow spawned from parent invoke state.
    ChildSpawned,
    /// Child workflow completed — parent wakes.
    ChildCompleted,
    /// Child workflow failed — supervision decision pending.
    ChildFailed,
    /// Child workflow restarted (supervision = restart).
    ChildRestarted,
    /// Instance cancelled by admin or API request.
    Cancelled,
}

// ---------------------------------------------------------------------------
// Actor types
// ---------------------------------------------------------------------------

/// Logical actor address: {instance_id}/{actor_path}.
/// Stable across restarts, migrations, scaling.
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ActorAddress {
    /// Instance that owns the actor tree.
    pub instance_id: Uuid,
    /// Dot-separated path from the root actor.
    pub actor_path: String,
}

impl ActorAddress {
    /// Create a root actor address for an instance.
    pub fn root(instance_id: Uuid) -> Self {
        Self {
            instance_id,
            actor_path: "root".to_string(),
        }
    }

    /// Create a child actor address.
    pub fn child(&self, name: &str) -> Self {
        Self {
            instance_id: self.instance_id,
            actor_path: format!("{}.{}", self.actor_path, name),
        }
    }
}

impl std::fmt::Display for ActorAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.instance_id, self.actor_path)
    }
}

/// Message priority for actor delivery.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Priority {
    /// Saga compensation, escalation actions.
    High = -1,
    /// Regular actor messages, state transitions.
    #[default]
    Normal = 0,
    /// Monitoring, metrics, non-critical notifications.
    Low = 1,
}

// ---------------------------------------------------------------------------
// Escalation
// ---------------------------------------------------------------------------

/// Escalation rules for a state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationRules {
    /// Duration in ms before sending warning.
    pub warn_after: Option<u64>,
    /// Duration in ms before escalating to supervisor.
    pub escalate_after: Option<u64>,
    /// Duration in ms before force-aborting.
    pub abort_after: Option<u64>,
}

// ---------------------------------------------------------------------------
// Migration
// ---------------------------------------------------------------------------

/// Migration plan for transitioning instances between workflow versions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationPlan {
    /// Definition version instances are migrated from.
    pub source_version: String,
    /// Definition version instances are migrated to.
    pub target_version: String,
    /// Old state ID → new state ID mapping.
    pub state_mappings: HashMap<String, String>,
    /// Old context field → new context field mapping.
    pub context_mappings: HashMap<String, String>,
}

// ---------------------------------------------------------------------------
// Audit
// ---------------------------------------------------------------------------

/// Audit entry for state transitions and admin actions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// When the action happened.
    pub timestamp: DateTime<Utc>,
    /// Namespace of the affected instance.
    pub namespace: NamespaceId,
    /// Affected instance.
    pub instance_id: Uuid,
    /// Action name (e.g. "transition", "suspend", "migrate").
    pub action: String,
    /// Identity that performed the action.
    pub actor: IdentityRef,
    /// Action-specific details.
    #[serde(default)]
    pub details: serde_json::Value,
}

// ---------------------------------------------------------------------------
// Usage counter (metering)
// ---------------------------------------------------------------------------

/// Usage counter for state change metering (standalone mode only).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageCounter {
    /// Period the counter covers.
    pub period: UsagePeriod,
    /// Number of state changes recorded in the period.
    pub state_changes: u64,
    /// Node that recorded the counter.
    pub node_id: String,
    /// HMAC signature for integrity verification.
    pub signature: Vec<u8>,
}

/// Usage counting period.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsagePeriod {
    /// Calendar year.
    pub year: u16,
    /// Calendar month (1-12).
    pub month: u8,
}

// ---------------------------------------------------------------------------
// Stream processing types
// ---------------------------------------------------------------------------

/// Stream source capability flags — declared per source type.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SourceCapabilities {
    /// Can seek to offset and replay (Kafka, NATS: yes. WebSocket, SSE: no).
    pub replayable: bool,
    /// Supports parallel consumption via partitions/queue groups.
    pub partitioned: bool,
    /// Supports pause/resume for flow control.
    pub back_pressurable: bool,
}

/// Fork mode — how trigger spawns children.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ForkMode {
    /// Lightweight ephemeral actor within parent (~10μs, zero DB write).
    /// Use for fast classify/filter/drop decisions.
    Actor,
    /// Independent durable workflow instance (~1-5ms with DB write).
    /// Use for complex processing (saga, human task, long-running).
    #[default]
    Instance,
}

/// Window type for stream aggregation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum WindowType {
    /// Time-based: items evicted when older than duration.
    Sliding {
        /// Window duration in milliseconds.
        duration_ms: u64,
    },
    /// Count-based: window fills → triggers evaluation → clears.
    Tumbling {
        /// Number of items per window.
        size: u64,
    },
    /// Gap-based: items grouped until silence exceeds gap duration.
    Session {
        /// Gap duration in milliseconds.
        gap_ms: u64,
    },
}

/// Drop policy when buffer overflows on replayable sources.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DropPolicy {
    /// Drop newest items, keep history (default).
    #[default]
    Newest,
    /// Drop oldest items, keep fresh data.
    Oldest,
}

/// Overflow strategy when buffer is full.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OverflowStrategy {
    /// Pause source consumption (requires back_pressurable source).
    #[default]
    SlowSource,
    /// Drop items (requires replayable source for safe replay).
    Drop,
    /// Auto-scale processing capacity (EE only, requires clustering).
    Scale,
}

/// Stream processing configuration — stored in workflow definition context
/// under `sflow:stream` key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamConfig {
    /// In-memory buffer size before applying overflow strategy.
    #[serde(default = "default_buffer_size")]
    pub buffer_size: u64,
    /// Maximum memory in MB for stream buffers.
    #[serde(default = "default_max_memory_mb")]
    pub max_memory_mb: u64,
    /// Strategy when buffer is full.
    #[serde(default)]
    pub overflow: OverflowStrategy,
    /// Which items to drop (only used when overflow = Drop and source is replayable).
    #[serde(default)]
    pub drop_policy: DropPolicy,
    /// Auto-scale processing (EE only).
    #[serde(default)]
    pub auto_scale: bool,
}

fn default_buffer_size() -> u64 {
    1000
}

fn default_max_memory_mb() -> u64 {
    256
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            buffer_size: default_buffer_size(),
            max_memory_mb: default_max_memory_mb(),
            overflow: OverflowStrategy::default(),
            drop_policy: DropPolicy::default(),
            auto_scale: false,
        }
    }
}

/// Stream source type identifier.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StreamSourceType {
    /// NATS JetStream consumer (CE, default transport).
    NatsJetstream,
    /// HTTP / SSE consumer (CE).
    HttpSse,
    /// gRPC server streaming (CE).
    GrpcStream,
    /// Kafka consumer (EE).
    Kafka,
    /// MQTT subscriber (EE, IoT).
    Mqtt,
    /// Custom WASM block implementing StreamSource WIT interface.
    Wasm,
}

/// Offset tracking for stream sources (source-specific).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StreamOffset {
    /// Kafka-style: partition + offset.
    Partitioned {
        /// Partition number.
        partition: u32,
        /// Offset within the partition.
        offset: u64,
    },
    /// NATS-style: sequence number.
    Sequence {
        /// Stream sequence number.
        seq: u64,
    },
    /// Opaque string offset (for custom sources).
    Opaque(String),
}

// ---------------------------------------------------------------------------
// Task Engine types
// ---------------------------------------------------------------------------

/// Task delivery mode — maps to NATS JetStream ack policies.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeliveryMode {
    /// Fire-and-forget. AckPolicy::None, MaxDeliver: 1.
    AtMostOnce,
    /// Retry until acknowledged. AckPolicy::Explicit (default).
    #[default]
    AtLeastOnce,
    /// Deduplication + idempotent. Nats-Msg-Id per task assignment.
    ExactlyOnce,
}

/// Task lifecycle status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    /// Queued for dispatch, no worker assigned yet.
    Queued,
    /// Assigned to a worker, execution in progress.
    Running,
    /// Completed successfully.
    Completed,
    /// Failed (may retry depending on policy).
    Failed,
    /// All retries exhausted, moved to dead letter queue.
    DeadLetter,
    /// Cancelled by admin or parent workflow.
    Cancelled,
}

/// Worker lifecycle status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkerStatus {
    /// Connected and accepting tasks.
    Active,
    /// Connected but paused (no new tasks).
    Paused,
    /// Draining — finishing current tasks, no new assignments.
    Draining,
    /// Disconnected (heartbeat expired).
    Disconnected,
}

/// Engine-to-worker commands sent via heartbeat stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkerCommand {
    /// Temporarily stop accepting new tasks.
    Pause,
    /// Resume accepting tasks.
    Resume,
    /// Finish current tasks and disconnect gracefully.
    Drain,
    /// Disconnect immediately (tasks will be redelivered).
    Shutdown,
}

/// Backoff function type for retry delays.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BackoffFunction {
    /// Constant delay between retries.
    Fixed,
    /// Delay increases linearly: base * attempt.
    Linear,
    /// Delay doubles each attempt: base * 2^attempt.
    #[default]
    Exponential,
    /// Delay follows fibonacci sequence: base * fib(attempt).
    Fibonacci,
    /// Custom schedule of delays.
    Custom,
}

/// Constraint operator for hard worker matching.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConstraintOperator {
    /// Equal.
    Eq,
    /// Not equal.
    Neq,
    /// Greater than.
    Gt,
    /// Greater than or equal.
    Gte,
    /// Less than.
    Lt,
    /// Less than or equal.
    Lte,
    /// Member of a list.
    In,
    /// Semantic version range match.
    Semver,
}

/// GPU capability of a worker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuCapability {
    /// GPU model name (e.g., "NVIDIA A100", "AMD MI250X").
    pub model: String,
    /// VRAM in MB.
    pub vram_mb: u64,
    /// Number of GPUs of this type.
    pub count: u32,
    /// CUDA compute capability (e.g., "8.0") or ROCm version.
    pub compute_version: String,
}

/// CPU capability of a worker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuCapability {
    /// Architecture (e.g., "x86_64", "aarch64").
    pub arch: String,
    /// Number of available cores.
    pub cores: u32,
    /// CPU model/family (e.g., "Intel Xeon E5", "Apple M2").
    pub model: String,
}

/// Memory capability of a worker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryCapability {
    /// Total memory in MB.
    pub total_mb: u64,
    /// Available memory in MB (updated via heartbeat).
    pub available_mb: u64,
}

/// Full capability declaration for a worker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerCapabilities {
    /// Worker identifier (stable across reconnects).
    pub worker_id: String,
    /// Queues this worker listens on.
    pub queues: Vec<String>,
    /// Maximum concurrent tasks.
    pub max_concurrent: u32,
    /// GPU capabilities (empty = no GPU).
    #[serde(default)]
    pub gpus: Vec<GpuCapability>,
    /// CPU capability.
    pub cpu: Option<CpuCapability>,
    /// Memory capability.
    pub memory: Option<MemoryCapability>,
    /// Installed packages/runtimes (e.g., {"ffmpeg": "6.0", "python": "3.11"}).
    #[serde(default)]
    pub packages: HashMap<String, String>,
    /// Custom labels (e.g., {"region": "eu-west-1", "tier": "premium"}).
    #[serde(default)]
    pub labels: HashMap<String, String>,
    /// SDK version (e.g., "sflow-worker-ts/1.0.0").
    pub sdk_version: String,
    /// OS + runtime info (e.g., "linux/amd64, Node.js 20.10").
    pub runtime_info: String,
}

/// Hard constraint for worker selection (MUST match).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraint {
    /// Attribute path (e.g., "labels.region", "packages.ffmpeg", "cpu.arch").
    pub attribute: String,
    /// Comparison operator.
    pub operator: ConstraintOperator,
    /// Value to compare against.
    pub value: String,
}

/// Soft affinity for worker scoring (PREFER matching).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Affinity {
    /// Attribute path.
    pub attribute: String,
    /// Value to prefer.
    pub value: String,
    /// Weight: positive = prefer, negative = anti-affinity. Range: -100..100.
    pub weight: i32,
}

/// Resource request for a single task.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceRequest {
    /// Required GPU VRAM in MB (0 = no GPU needed).
    #[serde(default)]
    pub gpu_vram_mb: u64,
    /// Required memory in MB (0 = no memory reservation).
    #[serde(default)]
    pub memory_mb: u64,
    /// Required CPU cores (0 = no CPU reservation).
    #[serde(default)]
    pub cpu_cores: u32,
}

/// Task requirements — what a task needs from a worker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRequirements {
    /// Queue name (NATS subject component).
    pub queue: String,
    /// Hard constraints (ALL must match).
    #[serde(default)]
    pub constraints: Vec<Constraint>,
    /// Soft affinities (weighted scoring, best-effort).
    #[serde(default)]
    pub affinities: Vec<Affinity>,
    /// Resource reservation for this task.
    #[serde(default)]
    pub resources: ResourceRequest,
    /// Delivery mode override (default: AtLeastOnce).
    #[serde(default)]
    pub delivery_mode: DeliveryMode,
    /// Task timeout in milliseconds (default: 30_000).
    #[serde(default = "default_task_timeout_ms")]
    pub timeout_ms: u64,
    /// Maximum retry attempts (default: 3).
    #[serde(default = "default_max_attempts")]
    pub max_attempts: u32,
    /// Backoff function between retries.
    #[serde(default)]
    pub backoff_function: BackoffFunction,
    /// Base delay in ms for backoff (default: 1000).
    #[serde(default = "default_backoff_base_ms")]
    pub backoff_base_ms: u64,
    /// Maximum backoff delay in ms cap.
    pub backoff_max_ms: Option<u64>,
    /// Custom backoff schedule in ms (for Custom function only).
    #[serde(default)]
    pub backoff_schedule: Vec<u64>,
}

fn default_task_timeout_ms() -> u64 {
    30_000
}

fn default_max_attempts() -> u32 {
    3
}

fn default_backoff_base_ms() -> u64 {
    1_000
}

/// A task assignment dispatched to a worker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskAssignment {
    /// Unique task ID.
    pub task_id: Uuid,
    /// Workflow instance that spawned this task.
    pub instance_id: Uuid,
    /// Namespace.
    pub namespace: NamespaceId,
    /// State that issued the invoke.
    pub state_id: String,
    /// Task queue name.
    pub queue: String,
    /// Input data for the task.
    pub input: serde_json::Value,
    /// Current attempt number (1-based).
    pub attempt: u32,
    /// Maximum attempts allowed.
    pub max_attempts: u32,
    /// Deadline for this attempt.
    pub deadline: DateTime<Utc>,
    /// Idempotency key for exactly-once delivery.
    pub idempotency_key: String,
    /// Delivery mode.
    pub delivery_mode: DeliveryMode,
    /// Metadata (e.g., correlation keys, tracing context).
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

/// Task result reported by worker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    /// Task ID.
    pub task_id: Uuid,
    /// Whether task succeeded.
    pub success: bool,
    /// Output data (on success).
    #[serde(default)]
    pub output: serde_json::Value,
    /// Error message (on failure).
    #[serde(default)]
    pub error: String,
    /// Error code (on failure, machine-readable).
    #[serde(default)]
    pub error_code: String,
    /// Worker that executed the task.
    pub worker_id: String,
    /// Execution duration in milliseconds.
    pub duration_ms: u64,
    /// Whether the error is retryable.
    #[serde(default)]
    pub retryable: bool,
}

/// Progress update from worker during task execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskProgress {
    /// Task ID.
    pub task_id: Uuid,
    /// Completion percentage (0-100).
    pub percent: u32,
    /// Human-readable status message.
    #[serde(default)]
    pub message: String,
    /// Structured progress data (task-specific).
    #[serde(default)]
    pub details: serde_json::Value,
}

/// Worker registration record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerRegistration {
    /// Stable worker identifier.
    pub worker_id: String,
    /// Full capability declaration.
    pub capabilities: WorkerCapabilities,
    /// Current status.
    pub status: WorkerStatus,
    /// Number of currently executing tasks.
    pub active_tasks: u32,
    /// Last heartbeat timestamp.
    pub last_heartbeat: DateTime<Utc>,
    /// Registration timestamp.
    pub registered_at: DateTime<Utc>,
    /// Worker hostname/IP for diagnostics.
    pub hostname: String,
}

/// Compensation definition for a saga step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompensationDef {
    /// Task queue for the compensation action.
    pub queue: String,
    /// Compensation input template (CEL expressions).
    #[serde(default)]
    pub input: serde_json::Value,
    /// Maximum retry attempts for compensation itself.
    #[serde(default = "default_max_attempts")]
    pub max_retries: u32,
    /// Timeout for compensation task in milliseconds.
    #[serde(default = "default_task_timeout_ms")]
    pub timeout_ms: u64,
}

// ---------------------------------------------------------------------------
// Nested Workflows
// ---------------------------------------------------------------------------

/// Supervision strategy for child workflows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SupervisionStrategy {
    /// Child fails → stop child, notify parent.
    Stop,
    /// Child fails → restart child with last known state.
    Restart,
    /// Child fails → propagate failure to parent.
    Escalate,
}

/// Configuration for child workflow dispatch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChildWorkflowConfig {
    /// Namespace for child (default: parent's namespace).
    pub namespace: Option<String>,
    /// Supervision strategy on child failure.
    #[serde(default = "default_supervision")]
    pub supervision: SupervisionStrategy,
    /// Max retries for restart supervision.
    #[serde(default)]
    pub max_retries: u32,
    /// Timeout in milliseconds for child completion.
    pub timeout_ms: Option<u64>,
    /// Correlation key expression (CEL).
    pub correlation_key: Option<String>,
    /// Child definition version (default: latest).
    pub definition_version: Option<String>,
}

fn default_supervision() -> SupervisionStrategy {
    SupervisionStrategy::Stop
}

// =============================================================================
// Flow Traits — Classification metadata per definition
// =============================================================================

/// How the workflow executes tasks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionMode {
    /// Caller blocks until the workflow completes.
    Sync,
    /// Caller receives an instance id and polls or subscribes.
    Async,
    /// Workflow emits results continuously.
    Streaming,
}

/// Idempotency enforcement mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IdempotencyMode {
    /// Every start creates a new instance.
    None,
    /// Starts with the same key resolve to the same instance.
    ByKey,
    /// At most one active instance per definition.
    Singleton,
}

/// Whether the workflow supports compensation/rollback.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Reversibility {
    /// No compensation possible.
    Irreversible,
    /// Every step has a compensating action.
    Compensable,
    /// Only some steps can be compensated.
    Partial,
    /// Rollback to the last checkpoint.
    Checkpoint,
}

/// Execution priority level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FlowPriority {
    /// Background work.
    Low,
    /// Default priority.
    Normal,
    /// Latency-sensitive work.
    High,
    /// Must run ahead of everything else.
    Critical,
}

/// Workflow category for classification and routing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FlowCategory {
    /// Coordinates several services.
    Orchestration,
    /// Distributed transaction with compensation.
    Saga,
    /// Linear data processing.
    Pipeline,
    /// Reacts to a single event type.
    EventHandler,
    /// Runs on a schedule.
    Scheduled,
    /// Consumes a continuous stream.
    Stream,
    /// Waits on human input.
    HumanTask,
}

/// Classification metadata attached to a workflow definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowTraits {
    /// How the workflow executes tasks.
    #[serde(default = "default_execution_mode")]
    pub execution: ExecutionMode,
    /// Idempotency enforcement mode.
    #[serde(default = "default_idempotency_mode")]
    pub idempotency: IdempotencyMode,
    /// CEL expression producing idempotency key (when mode = ByKey).
    pub idempotency_key_expr: Option<String>,
    /// Compensation support.
    #[serde(default = "default_reversibility")]
    pub reversibility: Reversibility,
    /// Delivery guarantee / consistency model.
    #[serde(default)]
    pub consistency: DeliveryMode,
    /// Execution priority.
    #[serde(default = "default_priority")]
    pub priority: FlowPriority,
    /// Workflow category.
    #[serde(default = "default_category")]
    pub category: FlowCategory,
}

fn default_execution_mode() -> ExecutionMode {
    ExecutionMode::Async
}
fn default_idempotency_mode() -> IdempotencyMode {
    IdempotencyMode::None
}
fn default_reversibility() -> Reversibility {
    Reversibility::Irreversible
}
fn default_priority() -> FlowPriority {
    FlowPriority::Normal
}
fn default_category() -> FlowCategory {
    FlowCategory::Orchestration
}

// =============================================================================
// Typed Workflows — JSON Schema for input/output/context validation
// =============================================================================

/// JSON Schema definitions for workflow I/O and context validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowSchema {
    /// JSON Schema for workflow input validation.
    pub input_schema: Option<serde_json::Value>,
    /// JSON Schema for workflow output validation.
    pub output_schema: Option<serde_json::Value>,
    /// JSON Schema for context shape validation.
    pub context_schema: Option<serde_json::Value>,
}

// =============================================================================
// Pipeline Mode — simplified linear workflow definition
// =============================================================================

/// Retry policy for a pipeline step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    /// Maximum retry attempts before the step fails.
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
    /// First retry delay as ISO 8601 duration.
    #[serde(default = "default_initial_backoff")]
    pub initial_backoff: String,
    /// Backoff multiplier applied per attempt.
    #[serde(default = "default_multiplier")]
    pub multiplier: f64,
    /// Upper bound on the retry delay as ISO 8601 duration.
    pub max_backoff: Option<String>,
}

fn default_max_retries() -> u32 {
    3
}
fn default_initial_backoff() -> String {
    "PT1S".to_string()
}
fn default_multiplier() -> f64 {
    2.0
}

/// A single step in a pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStep {
    /// Step name, unique within the pipeline.
    pub name: String,
    /// Block invoked by the step.
    pub block_ref: String,
    /// Block configuration.
    #[serde(default)]
    pub config: serde_json::Value,
    /// Retry policy for the step.
    pub retry: Option<RetryPolicy>,
    /// Timeout as ISO 8601 duration.
    pub timeout: Option<String>,
    /// CEL condition — step skipped if evaluates to false.
    pub condition: Option<String>,
}

/// A simplified pipeline definition — auto-converts to statechart.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineDefinition {
    /// Definition id.
    pub id: String,
    /// Definition version.
    #[serde(default = "default_version")]
    pub version: String,
    /// Steps in execution order.
    pub steps: Vec<PipelineStep>,
    /// Block invoked when a step fails after retries.
    pub error_handler: Option<String>,
    /// I/O and context schemas.
    pub schema: Option<WorkflowSchema>,
    /// Classification metadata.
    pub traits: Option<FlowTraits>,
    /// Namespace the definition is registered in.
    pub namespace: Option<String>,
}

fn default_version() -> String {
    "1.0.0".to_string()
}

// =============================================================================
// Block Wizard System
// =============================================================================

/// Wizard field type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WizardFieldType {
    /// Free text.
    String,
    /// Numeric input.
    Number,
    /// Toggle.
    Boolean,
    /// One of a fixed set of options.
    Select,
    /// Masked input stored as a secret.
    Secret,
    /// JSON document.
    Json,
    /// CEL expression.
    Cel,
    /// Templated string.
    Template,
}

/// A single field in a wizard step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WizardField {
    /// Config key the field writes to.
    pub key: String,
    /// Human-readable label.
    pub label: String,
    /// Input type.
    pub field_type: WizardFieldType,
    /// Whether the field must be filled.
    #[serde(default)]
    pub required: bool,
    /// Pre-filled value.
    pub default_value: Option<String>,
    /// Help text shown next to the field.
    pub description: Option<String>,
    /// CEL validation expression.
    pub validation_expr: Option<String>,
    /// Choices for `Select` fields.
    #[serde(default)]
    pub options: Vec<String>,
    /// Placeholder shown when empty.
    pub placeholder: Option<String>,
    /// CEL condition for visibility.
    pub visible_when: Option<String>,
}

/// A step/section in a wizard form.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WizardStep {
    /// Step title.
    pub title: String,
    /// Step description.
    pub description: Option<String>,
    /// Fields shown in the step.
    pub fields: Vec<WizardField>,
}

/// Complete wizard definition for a block's configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WizardDefinition {
    /// Block the wizard configures.
    pub block_id: String,
    /// Wizard title.
    pub title: String,
    /// Wizard description.
    pub description: Option<String>,
    /// Steps in display order.
    pub steps: Vec<WizardStep>,
}

// --- Block SDK Builder Helpers ---

impl WizardDefinition {
    /// Start building a wizard for the given block.
    pub fn builder(block_id: impl Into<String>, title: impl Into<String>) -> WizardBuilder {
        WizardBuilder {
            block_id: block_id.into(),
            title: title.into(),
            description: None,
            steps: Vec::new(),
        }
    }
}

/// Fluent builder for `WizardDefinition`.
pub struct WizardBuilder {
    block_id: String,
    title: String,
    description: Option<String>,
    steps: Vec<WizardStep>,
}

impl WizardBuilder {
    /// Set the wizard description.
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Add a step built with `WizardStep::builder()`.
    pub fn step(mut self, step: WizardStep) -> Self {
        self.steps.push(step);
        self
    }

    /// Finish building the wizard.
    pub fn build(self) -> WizardDefinition {
        WizardDefinition {
            block_id: self.block_id,
            title: self.title,
            description: self.description,
            steps: self.steps,
        }
    }
}

impl WizardStep {
    /// Start building a wizard step.
    pub fn builder(title: impl Into<String>) -> WizardStepBuilder {
        WizardStepBuilder {
            title: title.into(),
            description: None,
            fields: Vec::new(),
        }
    }
}

/// Fluent builder for `WizardStep`.
pub struct WizardStepBuilder {
    title: String,
    description: Option<String>,
    fields: Vec<WizardField>,
}

impl WizardStepBuilder {
    /// Set the step description.
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Add a field built with `WizardField::builder()`.
    pub fn field(mut self, field: WizardField) -> Self {
        self.fields.push(field);
        self
    }

    /// Finish building the step.
    pub fn build(self) -> WizardStep {
        WizardStep {
            title: self.title,
            description: self.description,
            fields: self.fields,
        }
    }
}

impl WizardField {
    /// Start building a wizard field.
    pub fn builder(
        key: impl Into<String>,
        label: impl Into<String>,
        field_type: WizardFieldType,
    ) -> WizardFieldBuilder {
        WizardFieldBuilder {
            key: key.into(),
            label: label.into(),
            field_type,
            required: false,
            default_value: None,
            description: None,
            validation_expr: None,
            options: Vec::new(),
            placeholder: None,
            visible_when: None,
        }
    }

    /// Shorthand: required string field.
    pub fn required_string(key: impl Into<String>, label: impl Into<String>) -> WizardField {
        Self::builder(key, label, WizardFieldType::String)
            .required()
            .build()
    }

    /// Shorthand: optional string field.
    pub fn optional_string(key: impl Into<String>, label: impl Into<String>) -> WizardField {
        Self::builder(key, label, WizardFieldType::String).build()
    }

    /// Shorthand: select field with options.
    pub fn select(
        key: impl Into<String>,
        label: impl Into<String>,
        options: Vec<String>,
    ) -> WizardField {
        Self::builder(key, label, WizardFieldType::Select)
            .options(options)
            .required()
            .build()
    }

    /// Shorthand: secret field (masked input).
    pub fn secret(key: impl Into<String>, label: impl Into<String>) -> WizardField {
        Self::builder(key, label, WizardFieldType::Secret)
            .required()
            .build()
    }

    /// Shorthand: boolean toggle.
    pub fn toggle(key: impl Into<String>, label: impl Into<String>) -> WizardField {
        Self::builder(key, label, WizardFieldType::Boolean).build()
    }
}

/// Fluent builder for `WizardField`.
pub struct WizardFieldBuilder {
    key: String,
    label: String,
    field_type: WizardFieldType,
    required: bool,
    default_value: Option<String>,
    description: Option<String>,
    validation_expr: Option<String>,
    options: Vec<String>,
    placeholder: Option<String>,
    visible_when: Option<String>,
}

impl WizardFieldBuilder {
    /// Mark the field as required.
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// Set the pre-filled value.
    pub fn default_value(mut self, val: impl Into<String>) -> Self {
        self.default_value = Some(val.into());
        self
    }

    /// Set the help text.
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set the CEL validation expression.
    pub fn validation(mut self, cel_expr: impl Into<String>) -> Self {
        self.validation_expr = Some(cel_expr.into());
        self
    }

    /// Set the choices for a `Select` field.
    pub fn options(mut self, opts: Vec<String>) -> Self {
        self.options = opts;
        self
    }

    /// Set the placeholder text.
    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = Some(text.into());
        self
    }

    /// Set the CEL visibility condition.
    pub fn visible_when(mut self, cel_expr: impl Into<String>) -> Self {
        self.visible_when = Some(cel_expr.into());
        self
    }

    /// Finish building the field.
    pub fn build(self) -> WizardField {
        WizardField {
            key: self.key,
            label: self.label,
            field_type: self.field_type,
            required: self.required,
            default_value: self.default_value,
            description: self.description,
            validation_expr: self.validation_expr,
            options: self.options,
            placeholder: self.placeholder,
            visible_when: self.visible_when,
        }
    }
}

// =============================================================================
// Block Port System — Typed ports for ComfyUI-style visual connections
// =============================================================================

/// Input port definition — named, typed connection point on a block.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputPortDef {
    /// Port name, unique within block (e.g., "document_image").
    pub name: String,
    /// Nominal type: "STRING", "INT", "FLOAT", "BOOL", "BINARY", "JSON", etc.
    pub port_type: String,
    /// Whether this port must be connected or have a default value.
    pub required: bool,
    /// Default value when port is not connected.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,
    /// Additional compatible types beyond exact port_type match.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub accepts: Vec<String>,
    /// Widget hint for inline UI editing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub widget: Option<WidgetHint>,
    /// JSON Schema for runtime value validation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<serde_json::Value>,
}

/// Output port definition — named, typed output connection point.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputPortDef {
    /// Port name, unique within block (e.g., "payment_id").
    pub name: String,
    /// Nominal type.
    pub port_type: String,
    /// Role: data (always produced) or conditional (only in specific outcomes).
    #[serde(default)]
    pub role: OutputRole,
    /// JSON Schema for runtime value validation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<serde_json::Value>,
}

/// Output port role.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputRole {
    /// Always produced on successful execution.
    #[default]
    Data,
    /// Produced only in specific outcomes (used for routing).
    Conditional,
}

/// Outcome definition — named execution scenario for visual editor routing.
/// Outcomes are metadata for the UI, not runtime constructs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutcomeDef {
    /// Outcome name (e.g., "success", "requires_3ds").
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// Which output port names are populated in this outcome.
    pub outputs: Vec<String>,
}

/// Widget hint for inline port configuration in the visual editor.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WidgetHint {
    /// Free-text input.
    Text {
        /// Placeholder shown when empty.
        #[serde(skip_serializing_if = "Option::is_none")]
        placeholder: Option<String>,
        /// Render as a multi-line editor.
        #[serde(default)]
        multiline: bool,
        /// Maximum character count.
        #[serde(skip_serializing_if = "Option::is_none")]
        max_length: Option<usize>,
    },
    /// Numeric input.
    Number {
        /// Minimum accepted value.
        #[serde(skip_serializing_if = "Option::is_none")]
        min: Option<f64>,
        /// Maximum accepted value.
        #[serde(skip_serializing_if = "Option::is_none")]
        max: Option<f64>,
        /// Increment step.
        #[serde(skip_serializing_if = "Option::is_none")]
        step: Option<f64>,
        /// Render as a slider instead of a text box.
        #[serde(default)]
        slider: bool,
    },
    /// Toggle.
    Boolean,
    /// Single choice from a list.
    Select {
        /// Available choices.
        options: Vec<SelectOption>,
    },
    /// Multiple choices from a list.
    MultiSelect {
        /// Available choices.
        options: Vec<SelectOption>,
        /// Maximum number of selected choices.
        #[serde(skip_serializing_if = "Option::is_none")]
        max: Option<usize>,
    },
    /// Colour picker.
    Color {
        /// Output format (e.g. "hex", "rgb").
        #[serde(default = "default_color_format")]
        format: String,
    },
    /// Code editor.
    Code {
        /// Syntax highlighting language.
        #[serde(default = "default_code_language")]
        language: String,
    },
    /// Masked secret input.
    Secret,
    /// File upload.
    File {
        /// Accepted MIME types or extensions.
        #[serde(skip_serializing_if = "Option::is_none")]
        accept: Option<String>,
        /// Maximum file size in megabytes.
        #[serde(skip_serializing_if = "Option::is_none")]
        max_size_mb: Option<u32>,
    },
    /// URL input.
    Url,
    /// Cron expression input.
    Cron,
    /// ISO 8601 duration input.
    Duration {
        /// Minimum accepted duration.
        #[serde(skip_serializing_if = "Option::is_none")]
        min: Option<String>,
        /// Maximum accepted duration.
        #[serde(skip_serializing_if = "Option::is_none")]
        max: Option<String>,
    },
    /// JSON editor.
    Json {
        /// JSON Schema the value must satisfy.
        #[serde(skip_serializing_if = "Option::is_none")]
        schema: Option<serde_json::Value>,
    },
}

/// Option for select/multi_select widgets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectOption {
    /// Text shown to the user.
    pub label: String,
    /// Value written to the port.
    pub value: serde_json::Value,
}

fn default_color_format() -> String {
    "hex".to_string()
}

fn default_code_language() -> String {
    "cel".to_string()
}

/// Block metadata with typed ports — returned by Block::metadata().
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockPortMetadata {
    /// Block identifier.
    pub id: String,
    /// Human-readable block name.
    pub name: String,
    /// Block version.
    pub version: String,
    /// Input port definitions.
    pub inputs: Vec<InputPortDef>,
    /// Output port definitions.
    pub outputs: Vec<OutputPortDef>,
    /// Named outcomes the block can finish with.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub outcomes: Vec<OutcomeDef>,
    /// JSON Schema for the block configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_schema: Option<serde_json::Value>,
}

/// Named port values — input to or output from a block.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PortValues(pub HashMap<String, serde_json::Value>);

impl PortValues {
    /// Empty port set.
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// Set a port value, replacing any previous value.
    pub fn insert(&mut self, name: impl Into<String>, value: serde_json::Value) {
        self.0.insert(name.into(), value);
    }

    /// Read a port value.
    pub fn get(&self, name: &str) -> Option<&serde_json::Value> {
        self.0.get(name)
    }

    /// Whether a port has a value.
    pub fn contains(&self, name: &str) -> bool {
        self.0.contains_key(name)
    }
}

/// Event emitted by a block to the parent statechart.
///
/// Maps to xstate v5 `sendParent()`. Engine delivers these as regular
/// events to the parent state, which can handle them via `on` transitions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockEvent {
    /// Event type name (e.g., "PROGRESS", "WARNING", "PARTIAL_RESULT").
    pub event_type: String,
    /// Event data payload.
    pub data: serde_json::Value,
}

/// Block execution result — collected outputs and events.
///
/// Blocks emit outputs and events through the OutputChannel during execution.
/// The engine collects them into this result after the block completes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockResult {
    /// Named output port values (last value per port from OutputChannel).
    pub outputs: PortValues,
    /// Events emitted to parent statechart during execution.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub events: Vec<BlockEvent>,
}

/// Output channel for blocks to emit results and events during execution.
///
/// Three execution patterns (matching xstate v5 invoke types):
/// - **Promise:** emit once, return (HTTP calls, transforms — ~90% of blocks)
/// - **Callback:** emit events over time (progress, warnings, interim results)
/// - **Observable:** stream values to output ports (LLM tokens, batch items)
///
/// Key invariants:
/// - Multiple emissions to same port: engine keeps LAST value for outputMapping
/// - Events are fire-and-forget: block does not see parent's reaction
/// - Progress is informational: no effect on transitions
pub trait OutputChannel: Send + Sync {
    /// Emit a value to a named output port.
    /// Can be called multiple times per port (streaming).
    fn emit(&self, port: &str, value: serde_json::Value) -> Result<(), crate::error::SflowError>;

    /// Emit an event to the parent statechart.
    /// Engine delivers as regular event (triggers `on` transitions).
    fn event(
        &self,
        event_type: &str,
        data: serde_json::Value,
    ) -> Result<(), crate::error::SflowError>;

    /// Report execution progress (0.0 to 1.0).
    fn progress(&self, fraction: f64) -> Result<(), crate::error::SflowError>;
}

/// Collector-based OutputChannel implementation.
///
/// Collects all emissions into vecs. After block completes, the engine
/// extracts outputs (last value per port) and events.
pub struct CollectorChannel {
    outputs: std::sync::Mutex<HashMap<String, serde_json::Value>>,
    events: std::sync::Mutex<Vec<BlockEvent>>,
    progress_value: std::sync::Mutex<f64>,
}

impl CollectorChannel {
    /// Empty collector.
    pub fn new() -> Self {
        Self {
            outputs: std::sync::Mutex::new(HashMap::new()),
            events: std::sync::Mutex::new(Vec::new()),
            progress_value: std::sync::Mutex::new(0.0),
        }
    }

    /// Extract collected results into a BlockResult.
    pub fn into_result(self) -> BlockResult {
        let outputs = self.outputs.into_inner().unwrap_or_default();
        let events = self.events.into_inner().unwrap_or_default();
        BlockResult {
            outputs: PortValues(outputs),
            events,
        }
    }

    /// Get the last reported progress value.
    pub fn progress_value(&self) -> f64 {
        *self
            .progress_value
            .lock()
            .unwrap_or_else(|e| e.into_inner())
    }
}

impl Default for CollectorChannel {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputChannel for CollectorChannel {
    fn emit(&self, port: &str, value: serde_json::Value) -> Result<(), crate::error::SflowError> {
        self.outputs
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(port.to_string(), value);
        Ok(())
    }

    fn event(
        &self,
        event_type: &str,
        data: serde_json::Value,
    ) -> Result<(), crate::error::SflowError> {
        self.events
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push(BlockEvent {
                event_type: event_type.to_string(),
                data,
            });
        Ok(())
    }

    fn progress(&self, fraction: f64) -> Result<(), crate::error::SflowError> {
        *self
            .progress_value
            .lock()
            .unwrap_or_else(|e| e.into_inner()) = fraction.clamp(0.0, 1.0);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wizard_builder_full() {
        let wizard = WizardDefinition::builder("stripe.payment", "Stripe Payment Setup")
            .description("Configure Stripe payment processing")
            .step(
                WizardStep::builder("Authentication")
                    .description("Stripe API credentials")
                    .field(WizardField::secret("api_key", "API Key"))
                    .field(
                        WizardField::builder("mode", "Mode", WizardFieldType::Select)
                            .options(vec!["test".into(), "live".into()])
                            .required()
                            .default_value("test")
                            .build(),
                    )
                    .build(),
            )
            .step(
                WizardStep::builder("Payment Options")
                    .field(WizardField::required_string("currency", "Currency"))
                    .field(WizardField::toggle(
                        "capture_immediately",
                        "Capture Immediately",
                    ))
                    .field(
                        WizardField::builder("webhook_url", "Webhook URL", WizardFieldType::String)
                            .placeholder("https://example.com/webhook")
                            .visible_when("mode == 'live'")
                            .build(),
                    )
                    .build(),
            )
            .build();

        assert_eq!(wizard.block_id, "stripe.payment");
        assert_eq!(wizard.title, "Stripe Payment Setup");
        assert!(wizard.description.is_some());
        assert_eq!(wizard.steps.len(), 2);

        let step0 = &wizard.steps[0];
        assert_eq!(step0.title, "Authentication");
        assert_eq!(step0.fields.len(), 2);
        assert_eq!(step0.fields[0].field_type, WizardFieldType::Secret);
        assert!(step0.fields[0].required);
        assert_eq!(step0.fields[1].field_type, WizardFieldType::Select);
        assert_eq!(step0.fields[1].options, vec!["test", "live"]);
        assert_eq!(step0.fields[1].default_value.as_deref(), Some("test"));

        let step1 = &wizard.steps[1];
        assert_eq!(step1.fields.len(), 3);
        assert!(step1.fields[0].required); // required_string
        assert!(!step1.fields[1].required); // toggle
        assert_eq!(step1.fields[1].field_type, WizardFieldType::Boolean);
        assert_eq!(
            step1.fields[2].visible_when.as_deref(),
            Some("mode == 'live'")
        );
    }

    #[test]
    fn test_wizard_shorthand_helpers() {
        let field = WizardField::required_string("name", "Name");
        assert!(field.required);
        assert_eq!(field.field_type, WizardFieldType::String);

        let field = WizardField::optional_string("notes", "Notes");
        assert!(!field.required);

        let field = WizardField::select(
            "env",
            "Environment",
            vec!["dev".into(), "staging".into(), "prod".into()],
        );
        assert!(field.required);
        assert_eq!(field.options.len(), 3);

        let field = WizardField::secret("token", "API Token");
        assert!(field.required);
        assert_eq!(field.field_type, WizardFieldType::Secret);

        let field = WizardField::toggle("enabled", "Enable Feature");
        assert!(!field.required);
        assert_eq!(field.field_type, WizardFieldType::Boolean);
    }

    #[test]
    fn test_wizard_field_builder_all_options() {
        let field = WizardField::builder("amount", "Amount", WizardFieldType::Number)
            .required()
            .default_value("100")
            .description("Payment amount in cents")
            .validation("value > 0 && value < 1000000")
            .placeholder("Enter amount...")
            .visible_when("payment_type == 'fixed'")
            .build();

        assert_eq!(field.key, "amount");
        assert_eq!(field.label, "Amount");
        assert_eq!(field.field_type, WizardFieldType::Number);
        assert!(field.required);
        assert_eq!(field.default_value.as_deref(), Some("100"));
        assert_eq!(
            field.description.as_deref(),
            Some("Payment amount in cents")
        );
        assert_eq!(
            field.validation_expr.as_deref(),
            Some("value > 0 && value < 1000000")
        );
        assert_eq!(field.placeholder.as_deref(), Some("Enter amount..."));
        assert_eq!(
            field.visible_when.as_deref(),
            Some("payment_type == 'fixed'")
        );
    }

    #[test]
    fn test_wizard_serialization_roundtrip() {
        let wizard = WizardDefinition::builder("test.block", "Test")
            .step(
                WizardStep::builder("Config")
                    .field(WizardField::required_string("url", "URL"))
                    .build(),
            )
            .build();

        let json = serde_json::to_string(&wizard).unwrap();
        let deserialized: WizardDefinition = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.block_id, "test.block");
        assert_eq!(deserialized.steps[0].fields[0].key, "url");
        assert!(deserialized.steps[0].fields[0].required);
    }
}
