use anyhow::{Context, Result};
use ed25519_dalek::VerifyingKey;
use std::env;

#[derive(Debug, Clone, PartialEq)]
pub enum TimeSource {
    /// Dev only — system clock, NOT audit-safe.
    System,
    /// Phase 2: Roughtime / NTP attestation stub.
    NtpSynced,
}

/// Which KMS backend to use for envelope encryption of trace_json.
#[derive(Debug, Clone, PartialEq)]
pub enum KmsProvider {
    /// KMS_PROVIDER unset — no encryption; plaintext mode (dev/legacy).
    None,
    /// KMS_PROVIDER=local — LocalDevProvider using a fixed 32-byte key (CI / local dev only).
    Local,
    /// KMS_PROVIDER=aws — AwsKmsProvider using AWS KMS CMK.
    Aws,
    /// KMS_PROVIDER=vault — VaultTransitProvider using HashiCorp Vault Transit secrets engine.
    Vault,
}

/// Which MTA client to instantiate at startup.
#[derive(Debug, Clone, PartialEq)]
pub enum MtaMode {
    /// Production: connect to a real MTA operator (MacroPulse or custom).
    /// Requires MTA_URL and MTA_PUBKEY_HEX.
    MacroPulse,
    /// Evaluation / CI: built-in mock that returns a static Expansion regime.
    /// No external endpoint required. Do NOT use in production.
    Mock,
    /// No external signal — IRL seals and audits every decision but applies
    /// no regime-level direction or notional constraints. Agent-level caps
    /// from the MAR are still enforced. All traces record signal_mode="none".
    ///
    /// Valid for production. Use when the firm manages risk externally (OMS,
    /// pre-trade risk checks) and wants IRL purely as a cryptographic audit rail.
    None,
}

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub mta_mode: MtaMode,
    /// Only used when mta_mode = MacroPulse.
    pub mta_url: String,
    /// Only used when mta_mode = MacroPulse.
    pub mta_pubkey: VerifyingKey,
    /// Valid bearer tokens, one per client/fund.
    pub irl_api_tokens: Vec<String>,
    pub time_source: TimeSource,
    /// Maximum age of a heartbeat before it is rejected (milliseconds).
    pub max_heartbeat_drift_ms: u64,
    /// When true, every /authorize request must carry a valid SignedHeartbeat.
    pub layer2_enabled: bool,
    /// Tolerance for quantity divergence in bind-execution (0.0001 = 0.01%).
    pub bind_size_tolerance: f64,
    /// How long before a PENDING trace is expired by the verifier worker (ms).
    pub trace_expiry_ms: u64,
    pub port: u16,
    /// When true, policy violations are logged but not blocked.
    /// Traces are persisted with policy_result = 'SHADOW_HALTED'.
    /// Safe for first-run instrumentation; set to false in production enforcement.
    pub shadow_mode: bool,
    /// When true, expose GET /metrics in Prometheus exposition format.
    pub metrics_enabled: bool,
    /// Maximum authorized requests per token per second (0 = disabled).
    /// Applies to all protected routes. Default: 100.
    pub rate_limit_per_second: u32,
    /// Maximum allowed request body size in bytes (0 = unlimited).
    /// Default: 1 MB (1_048_576 bytes). Protects against memory exhaustion.
    pub max_body_bytes: usize,
    /// KMS backend selection. None = plaintext mode.
    pub kms_provider: KmsProvider,
    /// CMK identifier: AWS key ARN/alias or Vault transit key name.
    /// Required when kms_provider is Aws or Vault.
    pub kms_key_id: Option<String>,
    /// Active key version used when generating new DEKs. Default: 1.
    pub kms_key_version: i32,
    /// When true, bind TLS listener via axum_server::bind_rustls.
    pub mtls_enabled: bool,
    /// When true, client certificate is required (not just accepted).
    pub mtls_required: bool,
    /// Path to server TLS certificate PEM file.
    pub tls_cert_path: Option<String>,
    /// Path to server TLS private key PEM file.
    pub tls_key_path: Option<String>,
    /// Path to CA certificate PEM used to verify client certs.
    pub tls_ca_cert_path: Option<String>,
    /// When true, generate ephemeral dev certs via rcgen (dev/CI only).
    pub mtls_dev_certs: bool,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        let database_url = env::var("DATABASE_URL").context("DATABASE_URL missing")?;

        let mta_mode = match env::var("MTA_MODE").as_deref() {
            Ok("mock") | Ok("Mock") => MtaMode::Mock,
            Ok("none") | Ok("None") => MtaMode::None,
            _ => MtaMode::MacroPulse,
        };

        // MTA credentials are only required when using a live MTA operator.
        let (mta_url, mta_pubkey) = if mta_mode == MtaMode::Mock {
            (String::new(), VerifyingKey::from_bytes(&[0u8; 32]).unwrap())
        } else {
            let url = env::var("MTA_URL").context("MTA_URL missing")?;
            let pubkey_hex = env::var("MTA_PUBKEY_HEX").context("MTA_PUBKEY_HEX missing")?;
            let pubkey_bytes =
                hex::decode(&pubkey_hex).context("MTA_PUBKEY_HEX is not valid hex")?;
            let pubkey_array: [u8; 32] = pubkey_bytes.try_into().map_err(|_| {
                anyhow::anyhow!("MTA_PUBKEY_HEX must be exactly 32 bytes (64 hex chars)")
            })?;
            let pubkey =
                VerifyingKey::from_bytes(&pubkey_array).context("Invalid Ed25519 public key")?;
            (url, pubkey)
        };

        let tokens_raw = env::var("IRL_API_TOKENS").context("IRL_API_TOKENS missing")?;
        let irl_api_tokens: Vec<String> = tokens_raw
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        anyhow::ensure!(
            !irl_api_tokens.is_empty(),
            "IRL_API_TOKENS must contain at least one token"
        );

        let time_source = match env::var("TIME_SOURCE").as_deref() {
            Ok("NtpSynced") => TimeSource::NtpSynced,
            _ => TimeSource::System,
        };

        let max_heartbeat_drift_ms = env::var("MAX_HEARTBEAT_DRIFT_MS")
            .unwrap_or_else(|_| "200".to_string())
            .parse::<u64>()
            .context("MAX_HEARTBEAT_DRIFT_MS must be a number")?;

        let layer2_enabled = env::var("LAYER2_ENABLED")
            .unwrap_or_else(|_| "true".to_string())
            .to_lowercase()
            == "true";

        let bind_size_tolerance = env::var("BIND_SIZE_TOLERANCE")
            .unwrap_or_else(|_| "0.0001".to_string())
            .parse::<f64>()
            .context("BIND_SIZE_TOLERANCE must be a float")?;

        let trace_expiry_ms = env::var("TRACE_EXPIRY_MS")
            .unwrap_or_else(|_| "3600000".to_string())
            .parse::<u64>()
            .context("TRACE_EXPIRY_MS must be a number")?;

        let port = env::var("PORT")
            .unwrap_or_else(|_| "4000".to_string())
            .parse::<u16>()
            .context("PORT must be a valid port number")?;

        let shadow_mode = env::var("SHADOW_MODE")
            .unwrap_or_else(|_| "false".to_string())
            .to_lowercase()
            == "true";

        let metrics_enabled = env::var("METRICS_ENABLED")
            .unwrap_or_else(|_| "true".to_string())
            .to_lowercase()
            == "true";

        let rate_limit_per_second = env::var("RATE_LIMIT_PER_SECOND")
            .unwrap_or_else(|_| "100".to_string())
            .parse::<u32>()
            .context("RATE_LIMIT_PER_SECOND must be a non-negative integer")?;

        let max_body_bytes = env::var("MAX_BODY_BYTES")
            .unwrap_or_else(|_| "1048576".to_string())
            .parse::<usize>()
            .context("MAX_BODY_BYTES must be a non-negative integer")?;

        let kms_provider = match env::var("KMS_PROVIDER").as_deref() {
            Ok("local") => KmsProvider::Local,
            Ok("aws") => KmsProvider::Aws,
            Ok("vault") => KmsProvider::Vault,
            _ => KmsProvider::None,
        };
        let kms_key_id = env::var("KMS_KEY_ID").ok();
        let kms_key_version = env::var("KMS_KEY_VERSION")
            .unwrap_or_else(|_| "1".to_string())
            .parse::<i32>()
            .context("KMS_KEY_VERSION must be an integer")?;

        let mtls_enabled = env::var("MTLS_ENABLED")
            .unwrap_or_else(|_| "false".to_string())
            .to_lowercase()
            == "true";
        let mtls_required = env::var("MTLS_REQUIRED")
            .unwrap_or_else(|_| "false".to_string())
            .to_lowercase()
            == "true";
        let tls_cert_path = env::var("TLS_CERT_PATH").ok();
        let tls_key_path = env::var("TLS_KEY_PATH").ok();
        let tls_ca_cert_path = env::var("TLS_CA_CERT_PATH").ok();
        let mtls_dev_certs = env::var("MTLS_DEV_CERTS")
            .unwrap_or_else(|_| "false".to_string())
            .to_lowercase()
            == "true";

        Ok(Config {
            database_url,
            mta_mode,
            mta_url,
            mta_pubkey,
            irl_api_tokens,
            time_source,
            max_heartbeat_drift_ms,
            layer2_enabled,
            bind_size_tolerance,
            trace_expiry_ms,
            port,
            shadow_mode,
            metrics_enabled,
            rate_limit_per_second,
            max_body_bytes,
            kms_provider,
            kms_key_id,
            kms_key_version,
            mtls_enabled,
            mtls_required,
            tls_cert_path,
            tls_key_path,
            tls_ca_cert_path,
            mtls_dev_certs,
        })
    }
}
