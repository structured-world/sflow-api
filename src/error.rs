//! SFLOW error types — public error model.

use std::collections::HashMap;

/// SFLOW error type — maps to google.rpc.Status + ErrorInfo.
#[derive(Debug, thiserror::Error)]
pub enum SflowError {
    #[error("instance not found: {instance_id}")]
    InstanceNotFound { instance_id: String },

    #[error("definition not found: {definition_id}")]
    DefinitionNotFound { definition_id: String },

    #[error("namespace not found: {namespace}")]
    NamespaceNotFound { namespace: String },

    #[error("invalid statechart: {reason}")]
    InvalidStatechart { reason: String },

    #[error("guard evaluation failed: {expression}")]
    GuardEvaluationFailed { expression: String },

    #[error("instance is suspended: {instance_id}")]
    InstanceSuspended { instance_id: String },

    #[error("instance already completed: {instance_id}")]
    InstanceCompleted { instance_id: String },

    #[error("quota exceeded: {detail}")]
    QuotaExceeded { detail: String },

    #[error("migration incompatible: {reason}")]
    MigrationIncompatible { reason: String },

    #[error("concurrent modification on instance: {instance_id}")]
    ConcurrentModification { instance_id: String },

    #[error("node unavailable: {node_id}")]
    NodeUnavailable { node_id: String },

    #[error("block execution failed: {block_id}: {reason}")]
    BlockExecutionFailed { block_id: String, reason: String },

    #[error("invalid CEL expression: {expression}: {reason}")]
    InvalidCelExpression { expression: String, reason: String },

    #[error("persistence error: {0}")]
    Persistence(String),

    #[error("transport error: {0}")]
    Transport(String),

    #[error("authentication error: {0}")]
    Authentication(String),

    #[error("authorization denied: {0}")]
    AuthorizationDenied(String),

    #[error("external service error ({service}): {message}")]
    ExternalServiceError { service: String, message: String },

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
