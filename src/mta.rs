use crate::config::Config;
use crate::errors::AppError;
use ed25519_dalek::{Signature, Verifier};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// The verified regime state produced by a Market Truth Anchor.
/// Every CognitiveSnapshot is anchored to an MtaState instance.
///
/// `regime_id` and `regime_label` are opaque — the MTA operator assigns them.
/// IRL's policy engine does not interpret them; it reads the three normalized
/// constraint fields that every operator must provide regardless of how their
/// model works internally.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MtaState {
    /// Opaque regime identifier assigned by the operator (0–255).
    pub regime_id: u8,
    /// Human-readable regime label — operator-defined, stored for audit.
    pub regime_label: String,
    /// Normalized risk level: 0.0 = fully defensive, 1.0 = fully risk-on.
    /// Derived from the operator's model; the policy engine uses this for
    /// context but relies on `allowed_sides` and `max_notional_scale` for
    /// hard enforcement.
    pub risk_level: f64,
    /// Regime-level notional multiplier (0.0–1.0) applied on top of the
    /// agent's per-profile cap.  Effective cap = agent_cap × max_notional_scale.
    pub max_notional_scale: f64,
    /// Trade directions permitted in this regime.
    /// Values: "long", "short", "neutral" (case-insensitive).
    /// An operator returning a 2-state bull/bear model, a VIX-based regime,
    /// or a continuous risk score all map to this same interface.
    pub allowed_sides: Vec<String>,
    /// Semantic version of the operator's model — stored in every trace.
    pub version: String,
    /// SHA-256 of the raw response body — proves which exact data was used.
    pub hash: String,
    /// Unix ms when the MTA broadcast this regime.
    pub broadcast_time: i64,
    /// Hex-encoded Ed25519 public key fingerprint of the operator that signed
    /// this broadcast. Stored in reasoning_traces.mta_pubkey_used (MTA-03).
    /// Used by authorize_agent to check agent's allowed_mta_pubkeys (MTA-02).
    pub pubkey_fingerprint: String,
    /// How this MTA state was produced. Stored in every trace for auditor transparency.
    /// Values: "live" (verified external signal), "none" (passthrough — no signal),
    /// "mock" (dev/CI evaluation only).
    pub signal_mode: String,
}

/// The interface any Market Truth Anchor operator must satisfy.
///
/// IRL is signal-agnostic: the engine depends on this trait, not on any
/// specific model or vendor. MacroPulse provides the reference implementation
/// ([`MacroPulseMtaClient`]), but any firm can bring its own regime signal —
/// a proprietary model, a third-party vendor, a multi-operator consensus
/// pipeline, or a simple rules-based classifier — by implementing this trait.
/// The seal, audit chain, and compliance guarantees are identical regardless
/// of the MTA source.
///
/// # Contract
/// The operator's model can be anything. The only requirement is translating
/// its output into the three normalized constraint fields:
/// - `risk_level` (0.0–1.0): how risk-on the current state is
/// - `max_notional_scale` (0.0–1.0): fraction of agent cap allowed now
/// - `allowed_sides`: which directions ("long", "short", "neutral") are open
///
/// The operator's internal methodology stays completely private.
///
/// # Example — custom implementation
/// ```rust,ignore
/// pub struct MyInternalMta { /* ... */ }
///
/// #[async_trait::async_trait]
/// impl MtaClient for MyInternalMta {
///     async fn fetch_verified(&self) -> Result<MtaState, AppError> {
///         // run your model, verify signature, map output to MtaState
///         // your methodology stays private — IRL only sees the signed MtaState
///     }
/// }
///
/// // In main.rs — one line to swap operators:
/// let mta_client: Arc<dyn MtaClient> = Arc::new(MyInternalMta::new(&config));
/// ```
#[async_trait::async_trait]
pub trait MtaClient: Send + Sync {
    /// Returns the current verified MTA state.
    ///
    /// Implementations are expected to cache results internally to minimise
    /// per-request latency. The engine calls this once per `/irl/authorize`
    /// request.
    async fn fetch_verified(&self) -> Result<MtaState, AppError>;
}

// ---------------------------------------------------------------------------
// MacroPulse managed MTA client
// ---------------------------------------------------------------------------

/// Raw response from MacroPulse's /v1/regime/current endpoint.
/// The `signature` field is Ed25519 over the canonical JSON of the other fields.
#[derive(Debug, Deserialize)]
struct RawMtaResponse {
    pub regime_id: u8,
    pub macro_regime: String,
    #[serde(rename = "model_version")]
    pub version: Option<String>,
    pub broadcast_time: i64,
    /// Base64-encoded Ed25519 signature over the response body (excluding this field).
    pub signature: String,
}

struct CachedMta {
    state: MtaState,
    fetched_at: Instant,
}

/// Fetches and verifies the current MacroPulse regime state.
///
/// This is the turnkey MtaClient implementation for the MacroPulse managed
/// MTA service. It connects to the MacroPulse broadcast endpoint, verifies
/// the Ed25519 signature against the pre-registered public key, and caches
/// the latest state for low-latency policy evaluation.
///
/// To use a custom MTA operator instead, implement the [`MtaClient`] trait
/// and pass it to [`AppState`] at startup.
pub struct MacroPulseMtaClient {
    http: reqwest::Client,
    mta_url: String,
    pubkey: ed25519_dalek::VerifyingKey,
    cache: Arc<RwLock<Option<CachedMta>>>,
}

/// How long a cached MTA response is considered fresh (milliseconds).
const CACHE_TTL_MS: u64 = 100;

/// How long a stale cached state can be used as a circuit-breaker fallback
/// when the MTA endpoint is unreachable. During this window trading continues
/// under the last known regime constraints; after it expires the engine fails
/// closed (all trading blocked) until MTA recovers.
const FALLBACK_TTL_SECS: u64 = 60;

impl MacroPulseMtaClient {
    pub fn new(cfg: &Config) -> Self {
        Self {
            http: reqwest::Client::builder()
                .timeout(Duration::from_millis(500))
                .build()
                .expect("Failed to build reqwest client"),
            mta_url: cfg.mta_url.clone(),
            pubkey: cfg.mta_pubkey,
            cache: Arc::new(RwLock::new(None)),
        }
    }

    async fn fetch_and_verify(&self) -> Result<MtaState, AppError> {
        let url = format!("{}/v1/regime/current", self.mta_url);

        let response = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::MtaFetchFailed(e.to_string()))?;

        let body_bytes = response
            .bytes()
            .await
            .map_err(|e| AppError::MtaFetchFailed(e.to_string()))?;

        let raw: RawMtaResponse = serde_json::from_slice(&body_bytes)
            .map_err(|e| AppError::MtaFetchFailed(format!("Invalid MTA JSON: {e}")))?;

        // Verify Ed25519 signature.
        // Signed payload = body bytes with the "signature" field stripped,
        // re-serialized as canonical JSON.
        self.verify_signature(&body_bytes, &raw)?;

        // Hash of the raw body — stored in every trace.
        let body_hash = {
            let mut hasher = Sha256::new();
            hasher.update(&body_bytes);
            hex::encode(hasher.finalize())
        };

        // Map MacroPulse's 4-regime HMM/PCA output to the normalized MtaState
        // constraint fields. This adapter is MacroPulse-specific; custom operators
        // produce these fields directly from their own model.
        let (risk_level, max_notional_scale, allowed_sides) = match raw.regime_id {
            0 => (1.00, 1.00, vec!["long", "short", "neutral"]), // expansion
            1 => (0.75, 0.75, vec!["long", "short", "neutral"]), // recovery
            2 => (0.30, 0.25, vec!["short", "neutral"]),         // tightening
            3 => (0.00, 0.00, vec!["neutral"]),                  // risk_off
            _ => (0.00, 0.00, vec!["neutral"]),                  // conservative default
        };

        Ok(MtaState {
            regime_id: raw.regime_id,
            regime_label: raw.macro_regime,
            risk_level,
            max_notional_scale,
            allowed_sides: allowed_sides.into_iter().map(String::from).collect(),
            version: raw.version.unwrap_or_else(|| "unknown".to_string()),
            hash: body_hash,
            broadcast_time: raw.broadcast_time,
            pubkey_fingerprint: hex::encode(self.pubkey.as_bytes()),
            signal_mode: "live".to_string(),
        })
    }

    fn verify_signature(&self, body_bytes: &[u8], raw: &RawMtaResponse) -> Result<(), AppError> {
        use base64::{engine::general_purpose::STANDARD, Engine};

        let sig_bytes = STANDARD
            .decode(&raw.signature)
            .map_err(|_| AppError::MtaSignatureInvalid)?;

        let sig_array: [u8; 64] = sig_bytes
            .try_into()
            .map_err(|_| AppError::MtaSignatureInvalid)?;

        let sig = Signature::from_bytes(&sig_array);

        // Reconstruct the canonical payload that MacroPulse signed:
        //   json.dumps(payload_without_signature, sort_keys=True, separators=(',', ':'))
        //
        // Parse the body, strip the "signature" key, re-serialize with sorted keys.
        let mut json_val: serde_json::Value =
            serde_json::from_slice(body_bytes).map_err(|_| AppError::MtaSignatureInvalid)?;

        if let Some(obj) = json_val.as_object_mut() {
            obj.remove("signature");
        }

        let canonical = canonical_json(&json_val).map_err(|_| AppError::MtaSignatureInvalid)?;

        self.pubkey
            .verify(canonical.as_bytes(), &sig)
            .map_err(|_| AppError::MtaSignatureInvalid)
    }
}

#[async_trait::async_trait]
impl MtaClient for MacroPulseMtaClient {
    /// Returns the current verified MTA state.
    /// Uses cached value if it is younger than `CACHE_TTL_MS`.
    async fn fetch_verified(&self) -> Result<MtaState, AppError> {
        // Fast path: return cached value if fresh
        {
            let cache = self.cache.read().await;
            if let Some(ref c) = *cache {
                if c.fetched_at.elapsed() < Duration::from_millis(CACHE_TTL_MS) {
                    return Ok(c.state.clone());
                }
            }
        }

        // Slow path: fetch, verify, cache
        match self.fetch_and_verify().await {
            Ok(state) => {
                let mut cache = self.cache.write().await;
                *cache = Some(CachedMta {
                    state: state.clone(),
                    fetched_at: Instant::now(),
                });
                Ok(state)
            }
            Err(e) => {
                // Circuit breaker: if the MTA endpoint is unreachable, use the
                // last known good state for up to FALLBACK_TTL_SECS seconds.
                // After the window expires the engine fails closed — trading
                // halts until MTA recovers, preventing unattested decisions.
                let cache = self.cache.read().await;
                if let Some(ref c) = *cache {
                    let age_secs = c.fetched_at.elapsed().as_secs();
                    if age_secs < FALLBACK_TTL_SECS {
                        tracing::warn!(
                            age_secs,
                            fallback_ttl = FALLBACK_TTL_SECS,
                            error = %e,
                            "MTA unreachable — using last known regime as circuit-breaker fallback"
                        );
                        return Ok(c.state.clone());
                    }
                    tracing::error!(
                        age_secs,
                        "MTA unreachable and fallback TTL expired ({FALLBACK_TTL_SECS}s) — failing closed"
                    );
                }
                Err(e)
            }
        }
    }
}

/// RFC 8785 canonical JSON: sorted object keys, no whitespace.
/// Must match Python's: json.dumps(obj, sort_keys=True, separators=(',', ':'))
fn canonical_json(value: &serde_json::Value) -> Result<String, ()> {
    use serde_json::Value;
    match value {
        Value::Object(map) => {
            let mut sorted: Vec<(&String, &Value)> = map.iter().collect();
            sorted.sort_by_key(|(k, _)| *k);
            let inner = sorted
                .into_iter()
                .map(|(k, v)| {
                    let key = serde_json::to_string(k).map_err(|_| ())?;
                    let val = canonical_json(v)?;
                    Ok(format!("{key}:{val}"))
                })
                .collect::<Result<Vec<_>, ()>>()?
                .join(",");
            Ok(format!("{{{inner}}}"))
        }
        Value::Array(arr) => {
            let inner = arr
                .iter()
                .map(canonical_json)
                .collect::<Result<Vec<_>, ()>>()?
                .join(",");
            Ok(format!("[{inner}]"))
        }
        other => serde_json::to_string(other).map_err(|_| ()),
    }
}

// ---------------------------------------------------------------------------
// Mock MTA client — for local evaluation and CI (not for production)
// ---------------------------------------------------------------------------

/// A built-in MTA implementation that returns a static Expansion (regime 0)
/// state without connecting to any external endpoint.
///
/// Enabled via `MTA_MODE=mock` in environment configuration. Allows firms to
/// run and evaluate the full IRL stack — authorize, bind, MAR, post-trade
/// verifier — without a live MTA operator or a MacroPulse subscription.
///
/// **Do not use in production.** The mock state is unsigned and trivially
/// reproducible; it provides no cryptographic attestation.
pub struct MockMtaClient;

#[async_trait::async_trait]
impl MtaClient for MockMtaClient {
    async fn fetch_verified(&self) -> Result<MtaState, AppError> {
        Ok(MtaState {
            regime_id: 0,
            regime_label: "expansion".to_string(),
            risk_level: 1.0,
            max_notional_scale: 1.0,
            allowed_sides: vec![
                "long".to_string(),
                "short".to_string(),
                "neutral".to_string(),
            ],
            version: "mock-v0".to_string(),
            hash: "mock0000000000000000000000000000000000000000000000000000000000000000"
                .to_string(),
            broadcast_time: chrono::Utc::now().timestamp_millis() - 50,
            pubkey_fingerprint: "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            signal_mode: "mock".to_string(),
        })
    }
}

// ---------------------------------------------------------------------------
// Null MTA client — no external signal, passthrough mode (MTA_MODE=none)
// ---------------------------------------------------------------------------

/// A production-valid MTA implementation that applies no regime constraints.
///
/// All trade directions are permitted and notional scale is 1.0. Agent-level
/// caps from the Multi-Agent Registry are still enforced by policy::enforce.
///
/// Use when the firm manages regime risk externally (OMS, pre-trade checks)
/// and wants IRL purely as a cryptographic audit and compliance rail.
///
/// Every trace records `signal_mode = "none"` so auditors can see that no
/// external signal was used. This is an intentional, documented production choice.
pub struct NullMtaClient;

#[async_trait::async_trait]
impl MtaClient for NullMtaClient {
    async fn fetch_verified(&self) -> Result<MtaState, AppError> {
        Ok(MtaState {
            regime_id: 0,
            regime_label: "unconstrained".to_string(),
            risk_level: 1.0,
            max_notional_scale: 1.0,
            allowed_sides: vec![
                "long".to_string(),
                "short".to_string(),
                "neutral".to_string(),
            ],
            version: "none".to_string(),
            hash: "none0000000000000000000000000000000000000000000000000000000000000000"
                .to_string(),
            broadcast_time: chrono::Utc::now().timestamp_millis(),
            pubkey_fingerprint: "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            signal_mode: "none".to_string(),
        })
    }
}

/// For unit tests: return a mock MTA state with specific constraints.
#[cfg(test)]
pub fn mock_mta(
    regime_id: u8,
    regime_label: &str,
    risk_level: f64,
    max_notional_scale: f64,
    allowed_sides: Vec<&str>,
) -> MtaState {
    MtaState {
        regime_id,
        regime_label: regime_label.to_string(),
        risk_level,
        max_notional_scale,
        allowed_sides: allowed_sides.into_iter().map(String::from).collect(),
        version: "test-v1".to_string(),
        hash: "0xmockhash".to_string(),
        broadcast_time: chrono::Utc::now().timestamp_millis() - 50,
        pubkey_fingerprint: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
        signal_mode: "mock".to_string(),
    }
}

#[cfg(test)]
mod custom_mta_tests {
    use super::*;

    /// Demonstrates that any type implementing MtaClient is accepted by the engine.
    /// This is the proof-of-concept for the "bring your own signal" guarantee.
    struct FirmInternalMta {
        regime_id: u8,
        regime_label: &'static str,
    }

    #[async_trait::async_trait]
    impl MtaClient for FirmInternalMta {
        async fn fetch_verified(&self) -> Result<MtaState, AppError> {
            // In production: run internal model, verify signature, map output → MtaState.
            // The firm's methodology is completely private — IRL only sees these fields.
            Ok(MtaState {
                regime_id: self.regime_id,
                regime_label: self.regime_label.to_string(),
                // Firm maps their model output to normalized constraint fields:
                risk_level: 0.3,
                max_notional_scale: 0.25,
                allowed_sides: vec!["short".to_string(), "neutral".to_string()],
                version: "firm-internal-v1".to_string(),
                hash: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                    .to_string(),
                broadcast_time: chrono::Utc::now().timestamp_millis() - 100,
                pubkey_fingerprint: "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".to_string(),
                signal_mode: "live".to_string(),
            })
        }
    }

    #[tokio::test]
    async fn custom_mta_client_satisfies_trait() {
        let client: Box<dyn MtaClient> = Box::new(FirmInternalMta {
            regime_id: 7, // opaque — firm-defined, no relation to MacroPulse taxonomy
            regime_label: "bear-squeeze",
        });
        let state = client.fetch_verified().await.unwrap();
        assert_eq!(state.regime_id, 7);
        assert_eq!(state.regime_label, "bear-squeeze");
        assert_eq!(state.version, "firm-internal-v1");
        assert_eq!(state.allowed_sides, vec!["short", "neutral"]);
    }

    #[tokio::test]
    async fn mock_mta_client_returns_expansion() {
        let client = MockMtaClient;
        let state = client.fetch_verified().await.unwrap();
        assert_eq!(state.regime_id, 0);
        assert_eq!(state.regime_label, "expansion");
        assert_eq!(state.risk_level, 1.0);
        assert_eq!(state.max_notional_scale, 1.0);
        assert!(state.allowed_sides.contains(&"long".to_string()));
    }

    #[tokio::test]
    async fn arc_dyn_dispatch_works() {
        // Verify Arc<dyn MtaClient> — the type used in AppState — dispatches correctly.
        let client: Arc<dyn MtaClient> = Arc::new(FirmInternalMta {
            regime_id: 42, // opaque — firm-defined
            regime_label: "defensive",
        });
        let state = client.fetch_verified().await.unwrap();
        assert_eq!(state.regime_id, 42);
        assert_eq!(state.max_notional_scale, 0.25);
    }

    #[tokio::test]
    async fn mta_state_includes_pubkey_fingerprint() {
        // MTA-03: MtaState must carry pubkey_fingerprint for audit storage.
        let client = MockMtaClient;
        let state = client.fetch_verified().await.unwrap();
        assert!(
            !state.pubkey_fingerprint.is_empty(),
            "pubkey_fingerprint must be populated"
        );
        assert_eq!(state.pubkey_fingerprint.len(), 64, "fingerprint must be 64 hex chars");
    }

    #[tokio::test]
    async fn firm_mta_pubkey_fingerprint_is_populated() {
        // MTA-02: custom operator MtaState carries a unique pubkey fingerprint.
        let client = FirmInternalMta { regime_id: 1, regime_label: "test" };
        let state = client.fetch_verified().await.unwrap();
        assert_eq!(state.pubkey_fingerprint.len(), 64);
        // Firm's fingerprint is distinct from MockMtaClient's (all-zeros)
        assert_ne!(
            state.pubkey_fingerprint,
            "0000000000000000000000000000000000000000000000000000000000000000"
        );
    }
}
