//! SFLOW error types — public error model.

use std::collections::HashMap;

/// SFLOW error type — maps to google.rpc.Status + ErrorInfo.
#[derive(Debug, thiserror::Error)]
pub enum SflowError {
    /// No workflow instance exists with the given id.
    #[error("instance not found: {instance_id}")]
    InstanceNotFound {
        /// Requested instance id.
        instance_id: String,
    },

    /// No workflow definition exists with the given id.
    #[error("definition not found: {definition_id}")]
    DefinitionNotFound {
        /// Requested definition id.
        definition_id: String,
    },

    /// The namespace is unknown to the engine.
    #[error("namespace not found: {namespace}")]
    NamespaceNotFound {
        /// Requested namespace name.
        namespace: String,
    },

    /// The statechart definition failed validation.
    #[error("invalid statechart: {reason}")]
    InvalidStatechart {
        /// Validation failure description.
        reason: String,
    },

    /// A transition guard could not be evaluated.
    #[error("guard evaluation failed: {expression}")]
    GuardEvaluationFailed {
        /// Guard expression that failed.
        expression: String,
    },

    /// The instance is suspended and rejects events.
    #[error("instance is suspended: {instance_id}")]
    InstanceSuspended {
        /// Suspended instance id.
        instance_id: String,
    },

    /// The instance already reached a final state.
    #[error("instance already completed: {instance_id}")]
    InstanceCompleted {
        /// Completed instance id.
        instance_id: String,
    },

    /// A namespace or tenant quota was exceeded.
    #[error("quota exceeded: {detail}")]
    QuotaExceeded {
        /// Which quota was exceeded.
        detail: String,
    },

    /// The migration plan cannot be applied to the instance.
    #[error("migration incompatible: {reason}")]
    MigrationIncompatible {
        /// Incompatibility description.
        reason: String,
    },

    /// Optimistic concurrency check failed on the instance.
    #[error("concurrent modification on instance: {instance_id}")]
    ConcurrentModification {
        /// Instance that was modified concurrently.
        instance_id: String,
    },

    /// A cluster node required for the operation is unreachable.
    #[error("node unavailable: {node_id}")]
    NodeUnavailable {
        /// Unreachable node id.
        node_id: String,
    },

    /// An invoked block returned an error.
    #[error("block execution failed: {block_id}: {reason}")]
    BlockExecutionFailed {
        /// Failing block id.
        block_id: String,
        /// Failure description reported by the block.
        reason: String,
    },

    /// A CEL expression failed to parse or type-check.
    #[error("invalid CEL expression: {expression}: {reason}")]
    InvalidCelExpression {
        /// Offending expression.
        expression: String,
        /// Parser or checker diagnostic.
        reason: String,
    },

    /// The persistence backend returned an error.
    #[error("persistence error: {0}")]
    Persistence(String),

    /// The event transport returned an error.
    #[error("transport error: {0}")]
    Transport(String),

    /// The caller could not be authenticated.
    #[error("authentication error: {0}")]
    Authentication(String),

    /// The caller is authenticated but not allowed to perform the action.
    #[error("authorization denied: {0}")]
    AuthorizationDenied(String),

    /// An external service invoked by the workflow failed.
    #[error("external service error ({service}): {message}")]
    ExternalServiceError {
        /// Name of the external service.
        service: String,
        /// Error message returned by the service.
        message: String,
    },

    /// Unexpected engine failure not covered by another variant.
    #[error("internal error: {0}")]
    Internal(String),
}

impl SflowError {
    /// Machine-readable error reason (maps to ErrorInfo.reason).
    pub fn reason(&self) -> &'static str {
        match self {
            Self::InstanceNotFound { .. } => "INSTANCE_NOT_FOUND",
            Self::DefinitionNotFound { .. } => "DEFINITION_NOT_FOUND",
            Self::NamespaceNotFound { .. } => "NAMESPACE_NOT_FOUND",
            Self::InvalidStatechart { .. } => "INVALID_STATECHART",
            Self::GuardEvaluationFailed { .. } => "GUARD_EVALUATION_FAILED",
            Self::InstanceSuspended { .. } => "INSTANCE_SUSPENDED",
            Self::InstanceCompleted { .. } => "INSTANCE_COMPLETED",
            Self::QuotaExceeded { .. } => "QUOTA_EXCEEDED",
            Self::MigrationIncompatible { .. } => "MIGRATION_INCOMPATIBLE",
            Self::ConcurrentModification { .. } => "CONCURRENT_MODIFICATION",
            Self::NodeUnavailable { .. } => "NODE_UNAVAILABLE",
            Self::BlockExecutionFailed { .. } => "BLOCK_EXECUTION_FAILED",
            Self::InvalidCelExpression { .. } => "INVALID_CEL_EXPRESSION",
            Self::ExternalServiceError { .. } => "EXTERNAL_SERVICE_ERROR",
            Self::Persistence(_) => "PERSISTENCE_ERROR",
            Self::Transport(_) => "TRANSPORT_ERROR",
            Self::Authentication(_) => "AUTHENTICATION_ERROR",
            Self::AuthorizationDenied(_) => "AUTHORIZATION_DENIED",
            Self::Internal(_) => "INTERNAL_ERROR",
        }
    }

    /// Error domain (always "sflow" per google.rpc.ErrorInfo contract).
    pub fn domain(&self) -> &'static str {
        "sflow"
    }

    /// Contextual metadata key-value pairs.
    pub fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        match self {
            Self::InstanceNotFound { instance_id }
            | Self::InstanceSuspended { instance_id }
            | Self::InstanceCompleted { instance_id }
            | Self::ConcurrentModification { instance_id } => {
                meta.insert("instance_id".to_string(), instance_id.clone());
            }
            Self::DefinitionNotFound { definition_id } => {
                meta.insert("definition_id".to_string(), definition_id.clone());
            }
            Self::NamespaceNotFound { namespace } => {
                meta.insert("namespace".to_string(), namespace.clone());
            }
            Self::GuardEvaluationFailed { expression }
            | Self::InvalidCelExpression { expression, .. } => {
                meta.insert("expression".to_string(), expression.clone());
            }
            Self::NodeUnavailable { node_id } => {
                meta.insert("node_id".to_string(), node_id.clone());
            }
            Self::BlockExecutionFailed { block_id, .. } => {
                meta.insert("block_id".to_string(), block_id.clone());
            }
            Self::ExternalServiceError { service, .. } => {
                meta.insert("service".to_string(), service.clone());
            }
            _ => {}
        }
        meta
    }
}
