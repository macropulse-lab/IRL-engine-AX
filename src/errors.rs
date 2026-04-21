use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Bitemporal violation: valid_time must be strictly before txn_time")]
    BiTemporalViolation,

    #[error("MTA signature invalid: operator response cannot be trusted")]
    MtaSignatureInvalid,

    #[error("MTA fetch failed: {0}")]
    MtaFetchFailed(String),

    #[error("Heartbeat error: {0}")]
    Heartbeat(#[from] HeartbeatError),

    #[error("Policy violation: {0}")]
    Policy(#[from] PolicyError),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Trace not found: {0}")]
    TraceNotFound(String),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Trace {0} is already bound (status: {1})")]
    TraceAlreadyBound(String, String),

    #[error("Encryption error")]
    Encryption(String),

    #[error("Forbidden")]
    Forbidden,
}

#[derive(Debug, Error)]
pub enum HeartbeatError {
    #[error("Stale sequence: received {received}, last accepted {last}")]
    StaleSequence { received: u64, last: u64 },

    #[error("Heartbeat too old: drift {drift_ms}ms exceeds maximum {max_ms}ms")]
    LatencyThresholdExceeded { drift_ms: u64, max_ms: u64 },

    #[error("Heartbeat signature invalid")]
    InvalidSignature,

    #[error("Heartbeat required but not provided (LAYER2_ENABLED=true)")]
    Missing,

    #[error("Heartbeat mta_ref '{got}' does not match current MTA hash '{expected}'")]
    MtaRefMismatch { got: String, expected: String },
}

#[derive(Debug, Error)]
pub enum PolicyError {
    #[error("Regime violation: action '{action}' is prohibited in regime '{regime}' by policy '{policy}' v{version}")]
    RegimeViolation {
        action: String,
        regime: String,
        policy: String,
        version: String,
    },
    #[error(
        "Notional {notional:.2} exceeds regime '{regime}' limit {limit:.2} (policy '{policy}')"
    )]
    NotionalExceedsLimit {
        notional: f64,
        limit: f64,
        regime: String,
        policy: String,
    },
    #[error("Agent not found in registry")]
    AgentNotFound,
    #[error("Model hash mismatch: running agent does not match registered model hash")]
    ModelHashMismatch,
    #[error("Agent is not permitted to trade in regime {regime_id}")]
    RegimeUnauthorized { regime_id: u8 },
    #[error("Agent status is not Active")]
    AgentNotActive,
    #[error("MTA operator pubkey '{pubkey}' is not in agent's allowed_mta_pubkeys")]
    MtaPubkeyUnauthorized { pubkey: String },
}

impl AppError {
    /// Return the machine-readable error code string used in JSON responses
    /// and Prometheus metric labels.
    pub fn error_code(&self) -> &'static str {
        match self {
            AppError::BiTemporalViolation => "BITEMPORAL_VIOLATION",
            AppError::MtaSignatureInvalid => "MTA_SIGNATURE_INVALID",
            AppError::MtaFetchFailed(_) => "MTA_FETCH_FAILED",
            AppError::Heartbeat(HeartbeatError::Missing) => "HEARTBEAT_MISSING",
            AppError::Heartbeat(HeartbeatError::StaleSequence { .. }) => "HEARTBEAT_STALE_SEQUENCE",
            AppError::Heartbeat(HeartbeatError::LatencyThresholdExceeded { .. }) => {
                "HEARTBEAT_DRIFT_EXCEEDED"
            }
            AppError::Heartbeat(HeartbeatError::InvalidSignature) => "HEARTBEAT_SIGNATURE_INVALID",
            AppError::Heartbeat(HeartbeatError::MtaRefMismatch { .. }) => {
                "HEARTBEAT_MTA_REF_MISMATCH"
            }
            AppError::Policy(PolicyError::RegimeViolation { .. }) => "REGIME_VIOLATION",
            AppError::Policy(PolicyError::NotionalExceedsLimit { .. }) => "NOTIONAL_EXCEEDS_LIMIT",
            AppError::Policy(PolicyError::AgentNotFound) => "AGENT_NOT_FOUND",
            AppError::Policy(PolicyError::ModelHashMismatch) => "MODEL_HASH_MISMATCH",
            AppError::Policy(PolicyError::RegimeUnauthorized { .. }) => "REGIME_UNAUTHORIZED",
            AppError::Policy(PolicyError::AgentNotActive) => "AGENT_NOT_ACTIVE",
            AppError::Policy(PolicyError::MtaPubkeyUnauthorized { .. }) => "MTA_PUBKEY_UNAUTHORIZED",
            AppError::Database(_) => "DATABASE_ERROR",
            AppError::Serialization(_) => "SERIALIZATION_ERROR",
            AppError::TraceNotFound(_) => "TRACE_NOT_FOUND",
            AppError::Unauthorized => "UNAUTHORIZED",
            AppError::RateLimitExceeded => "RATE_LIMIT_EXCEEDED",
            AppError::TraceAlreadyBound(_, _) => "TRACE_ALREADY_BOUND",
            AppError::Encryption(_) => "ENCRYPTION_ERROR",
            AppError::Forbidden => "FORBIDDEN",
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_code, message) = match &self {
            AppError::BiTemporalViolation => (
                StatusCode::BAD_REQUEST,
                "BITEMPORAL_VIOLATION",
                self.to_string(),
            ),
            AppError::MtaSignatureInvalid => (
                StatusCode::FORBIDDEN,
                "MTA_SIGNATURE_INVALID",
                self.to_string(),
            ),
            AppError::MtaFetchFailed(_) => (
                StatusCode::BAD_GATEWAY,
                "MTA_FETCH_FAILED",
                self.to_string(),
            ),
            AppError::Heartbeat(HeartbeatError::Missing) => (
                StatusCode::BAD_REQUEST,
                "HEARTBEAT_MISSING",
                self.to_string(),
            ),
            AppError::Heartbeat(HeartbeatError::StaleSequence { .. }) => (
                StatusCode::BAD_REQUEST,
                "HEARTBEAT_STALE_SEQUENCE",
                self.to_string(),
            ),
            AppError::Heartbeat(HeartbeatError::LatencyThresholdExceeded { .. }) => (
                StatusCode::BAD_REQUEST,
                "HEARTBEAT_DRIFT_EXCEEDED",
                self.to_string(),
            ),
            AppError::Heartbeat(HeartbeatError::InvalidSignature) => (
                StatusCode::FORBIDDEN,
                "HEARTBEAT_SIGNATURE_INVALID",
                self.to_string(),
            ),
            AppError::Heartbeat(HeartbeatError::MtaRefMismatch { .. }) => (
                StatusCode::FORBIDDEN,
                "HEARTBEAT_MTA_REF_MISMATCH",
                self.to_string(),
            ),
            AppError::Policy(PolicyError::RegimeViolation { .. }) => {
                (StatusCode::FORBIDDEN, "REGIME_VIOLATION", self.to_string())
            }
            AppError::Policy(PolicyError::NotionalExceedsLimit { .. }) => (
                StatusCode::FORBIDDEN,
                "NOTIONAL_EXCEEDS_LIMIT",
                self.to_string(),
            ),
            AppError::Policy(PolicyError::AgentNotFound) => {
                (StatusCode::NOT_FOUND, "AGENT_NOT_FOUND", self.to_string())
            }
            AppError::Policy(PolicyError::ModelHashMismatch) => (
                StatusCode::FORBIDDEN,
                "MODEL_HASH_MISMATCH",
                self.to_string(),
            ),
            AppError::Policy(PolicyError::RegimeUnauthorized { .. }) => (
                StatusCode::FORBIDDEN,
                "REGIME_UNAUTHORIZED",
                self.to_string(),
            ),
            AppError::Policy(PolicyError::AgentNotActive) => {
                (StatusCode::FORBIDDEN, "AGENT_NOT_ACTIVE", self.to_string())
            }
            AppError::Policy(PolicyError::MtaPubkeyUnauthorized { .. }) => (
                StatusCode::FORBIDDEN,
                "MTA_PUBKEY_UNAUTHORIZED",
                self.to_string(),
            ),
            AppError::Database(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "DATABASE_ERROR",
                "Internal storage error".to_string(),
            ),
            AppError::Serialization(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "SERIALIZATION_ERROR",
                "Internal serialization error".to_string(),
            ),
            AppError::TraceNotFound(id) => (
                StatusCode::NOT_FOUND,
                "TRACE_NOT_FOUND",
                format!("Trace not found: {id}"),
            ),
            AppError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "UNAUTHORIZED",
                "Invalid or missing bearer token".to_string(),
            ),
            AppError::RateLimitExceeded => (
                StatusCode::TOO_MANY_REQUESTS,
                "RATE_LIMIT_EXCEEDED",
                "Rate limit exceeded".to_string(),
            ),
            AppError::TraceAlreadyBound(id, status) => (
                StatusCode::CONFLICT,
                "TRACE_ALREADY_BOUND",
                format!("Trace {id} is already bound (status: {status})"),
            ),
            AppError::Encryption(detail) => {
                tracing::error!("Encryption error (internal): {detail}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "ENCRYPTION_ERROR",
                    "Internal encryption error".to_string(),
                )
            }
            AppError::Forbidden => (
                StatusCode::FORBIDDEN,
                "FORBIDDEN",
                "Forbidden: insufficient permissions".to_string(),
            ),
        };

        let body = Json(json!({
            "error": error_code,
            "message": message,
        }));

        (status, body).into_response()
    }
}
