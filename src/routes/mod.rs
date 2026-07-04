pub mod admin;
pub mod agents;
pub mod authorize;
pub mod bind;
pub mod tokens;
pub mod traces;

use crate::db;
use crate::errors::AppError;
use crate::metrics;
use crate::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

/// GET /
///
/// HTML landing page for the demo/sandbox instance.
/// Links to Swagger UI, health endpoint, and documentation.
pub async fn landing() -> Html<&'static str> {
    Html(LANDING_HTML)
}

const LANDING_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>IRL Engine — Developer Sandbox</title>
<meta name="description" content="IRL Engine developer sandbox. Interactive API, OpenAPI spec, and quick-start guide for the cryptographic pre-execution compliance gateway.">
<meta name="robots" content="noindex">
<link rel="preconnect" href="https://fonts.googleapis.com">
<link href="https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600;700&family=JetBrains+Mono:wght@400;500&display=swap" rel="stylesheet">
<style>
  *, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }
  :root {
    --bg: #090909; --s1: #0f0f0f; --s2: #141414;
    --border: #1a1a1a; --border2: #252525;
    --text: #f0f0f0; --muted: #888; --dim: #444;
    --green: #22c55e; --amber: #f59e0b;
  }
  html { scroll-behavior: smooth; }
  body { background: var(--bg); color: var(--text); font-family: 'Inter', -apple-system, sans-serif; line-height: 1.6; }
  a { color: inherit; text-decoration: none; }
  .mono { font-family: 'JetBrains Mono', monospace; }

  /* Nav */
  nav {
    position: fixed; top: 0; left: 0; right: 0; z-index: 100;
    height: 56px; display: flex; align-items: center; justify-content: space-between;
    padding: 0 2rem; border-bottom: 1px solid var(--border);
    background: rgba(9,9,9,0.92); backdrop-filter: blur(16px);
  }
  .nav-brand { font-size: 0.9rem; font-weight: 600; letter-spacing: -0.01em; display: flex; align-items: center; gap: 0.5rem; color: var(--muted); }
  .nav-brand strong { color: var(--text); }
  .nav-dot { width: 7px; height: 7px; border-radius: 50%; background: var(--green); box-shadow: 0 0 7px var(--green); }
  .nav-links { display: flex; align-items: center; gap: 1.75rem; }
  .nav-links a { font-size: 0.82rem; color: var(--muted); transition: color 0.2s; }
  .nav-links a:hover { color: var(--text); }
  .nav-links a.primary { color: var(--green); font-weight: 500; }

  /* Layout */
  .container { max-width: 860px; margin: 0 auto; padding: 0 2rem; }

  /* Hero */
  .hero { padding: 8rem 0 4rem; border-bottom: 1px solid var(--border); }
  .hero-status {
    display: inline-flex; align-items: center; gap: 0.5rem;
    font-size: 0.75rem; color: var(--muted); background: var(--s1);
    border: 1px solid var(--border2); padding: 0.3rem 0.75rem;
    border-radius: 4px; margin-bottom: 1.75rem; font-family: 'JetBrains Mono', monospace;
  }
  .status-dot { width: 6px; height: 6px; border-radius: 50%; background: var(--green); box-shadow: 0 0 5px var(--green); animation: pulse 2s infinite; }
  @keyframes pulse { 0%, 100% { opacity: 1; } 50% { opacity: 0.4; } }
  .hero h1 { font-size: clamp(1.75rem, 4vw, 2.75rem); font-weight: 700; letter-spacing: -0.03em; line-height: 1.15; margin-bottom: 1rem; }
  .hero h1 span { color: var(--green); }
  .hero-sub { font-size: 0.95rem; color: var(--muted); max-width: 560px; line-height: 1.7; margin-bottom: 2rem; }
  .hero-actions { display: flex; gap: 0.75rem; flex-wrap: wrap; }
  .btn-primary { display: inline-block; padding: 0.6rem 1.25rem; background: var(--green); color: #000; border-radius: 6px; font-size: 0.85rem; font-weight: 600; transition: opacity 0.2s; }
  .btn-primary:hover { opacity: 0.85; }
  .btn-ghost { display: inline-block; padding: 0.6rem 1.25rem; border: 1px solid var(--border2); color: var(--muted); border-radius: 6px; font-size: 0.85rem; font-weight: 500; transition: border-color 0.2s, color 0.2s; }
  .btn-ghost:hover { border-color: var(--muted); color: var(--text); }

  /* Quick start */
  .section { padding: 3.5rem 0; border-top: 1px solid var(--border); }
  .section-label { font-size: 0.7rem; color: var(--green); letter-spacing: 0.1em; text-transform: uppercase; font-weight: 600; margin-bottom: 0.75rem; }
  .section h2 { font-size: 1.25rem; font-weight: 700; letter-spacing: -0.02em; margin-bottom: 0.5rem; }
  .section-sub { font-size: 0.85rem; color: var(--muted); margin-bottom: 1.5rem; line-height: 1.65; }
  .steps { display: flex; flex-direction: column; gap: 0.75rem; }
  .step { display: flex; gap: 1rem; align-items: flex-start; }
  .step-num { width: 22px; height: 22px; border-radius: 50%; background: var(--s1); border: 1px solid var(--border2); display: flex; align-items: center; justify-content: center; font-size: 0.7rem; font-weight: 600; color: var(--muted); flex-shrink: 0; margin-top: 2px; }
  .step-body { flex: 1; }
  .step-title { font-size: 0.85rem; font-weight: 600; margin-bottom: 0.4rem; }
  .step-desc { font-size: 0.8rem; color: var(--muted); margin-bottom: 0.5rem; }
  pre {
    background: var(--s1); border: 1px solid var(--border2); border-radius: 6px;
    padding: 0.9rem 1rem; font-family: 'JetBrains Mono', monospace;
    font-size: 0.75rem; overflow-x: auto; line-height: 1.6;
    color: var(--text);
  }
  .comment { color: var(--dim); }
  .key { color: #7dd3fc; }
  .val { color: #86efac; }
  .str { color: #fca5a5; }
  .url { color: #c4b5fd; }

  /* API cards */
  .cards { display: grid; grid-template-columns: repeat(auto-fit, minmax(240px, 1fr)); gap: 0.75rem; }
  .card {
    background: var(--s1); border: 1px solid var(--border2); border-radius: 8px;
    padding: 1.25rem 1.5rem; text-decoration: none; color: inherit;
    transition: border-color 0.2s;
  }
  .card:hover { border-color: rgba(34,197,94,0.35); }
  .card-method { font-family: 'JetBrains Mono', monospace; font-size: 0.65rem; font-weight: 600; letter-spacing: 0.06em; text-transform: uppercase; color: var(--green); margin-bottom: 0.3rem; }
  .card h3 { font-size: 0.9rem; font-weight: 600; margin-bottom: 0.3rem; }
  .card p { font-size: 0.78rem; color: var(--muted); line-height: 1.5; }
  .card-path { font-family: 'JetBrains Mono', monospace; font-size: 0.7rem; color: var(--dim); margin-top: 0.5rem; }

  /* Demo agents */
  .agents-table { width: 100%; border-collapse: collapse; font-size: 0.8rem; }
  .agents-table th { text-align: left; padding: 0.5rem 1rem; border-bottom: 1px solid var(--border2); font-size: 0.68rem; color: var(--muted); letter-spacing: 0.06em; text-transform: uppercase; font-weight: 500; }
  .agents-table td { padding: 0.55rem 1rem; border-bottom: 1px solid var(--border); font-family: 'JetBrains Mono', monospace; font-size: 0.75rem; }
  .agents-table tr:last-child td { border-bottom: none; }
  .agents-table td:first-child { color: var(--muted); font-family: 'Inter', sans-serif; font-size: 0.8rem; }
  .tag-active { display: inline-block; background: rgba(34,197,94,0.12); color: var(--green); font-size: 0.65rem; font-weight: 600; padding: 0.1rem 0.4rem; border-radius: 3px; letter-spacing: 0.04em; }
  .agents-wrap { border: 1px solid var(--border2); border-radius: 8px; overflow: hidden; background: var(--s1); }

  /* Footer */
  footer { border-top: 1px solid var(--border); padding: 1.5rem 2rem; display: flex; align-items: center; justify-content: space-between; flex-wrap: wrap; gap: 1rem; }
  .footer-brand { font-size: 0.78rem; color: var(--dim); }
  .footer-links { display: flex; gap: 1.5rem; }
  .footer-links a { font-size: 0.78rem; color: var(--dim); transition: color 0.15s; }
  .footer-links a:hover { color: var(--muted); }

  @media (max-width: 640px) {
    nav { padding: 0 1rem; }
    .container { padding: 0 1.25rem; }
    .hero { padding: 6rem 0 3rem; }
    footer { flex-direction: column; align-items: flex-start; }
  }
</style>
</head>
<body>

<nav>
  <div class="nav-brand">
    <svg width="20" height="20" viewBox="0 0 100 100" fill="none" style="flex-shrink:0" aria-label="MacroPulse"><defs><clipPath id="sb-cr"><circle cx="54" cy="50" r="32"/></clipPath><mask id="sb-lmr"><circle cx="44" cy="50" r="32" fill='#fff'/><circle cx="54" cy="50" r="32" fill='#000'/></mask></defs><g fill='#3fb85a'><g mask="url(#sb-lmr)"><rect x="0" y="13.5" width="100" height="4.9"/><rect x="0" y="22.5" width="100" height="4.9"/><rect x="0" y="31.5" width="100" height="4.9"/><rect x="0" y="40.5" width="100" height="4.9"/><rect x="0" y="49.5" width="100" height="4.9"/><rect x="0" y="58.5" width="100" height="4.9"/><rect x="0" y="67.5" width="100" height="4.9"/><rect x="0" y="76.5" width="100" height="4.9"/><rect x="0" y="85.5" width="100" height="4.9"/></g><g clip-path="url(#sb-cr)"><rect x="0" y="9.0" width="100" height="4.9"/><rect x="0" y="18.0" width="100" height="4.9"/><rect x="0" y="27.0" width="100" height="4.9"/><rect x="0" y="36.0" width="100" height="4.9"/><rect x="0" y="45.0" width="100" height="4.9"/><rect x="0" y="54.0" width="100" height="4.9"/><rect x="0" y="63.0" width="100" height="4.9"/><rect x="0" y="72.0" width="100" height="4.9"/><rect x="0" y="81.0" width="100" height="4.9"/><rect x="0" y="90.0" width="100" height="4.9"/></g></g></svg>
    MacroPulse · <strong>IRL Engine</strong>
  </div>
  <div class="nav-links">
    <a href="https://macropulse.live/irl">IRL Overview</a>
    <a href="https://macropulse.live/irl-whitepaper">Whitepaper</a>
    <a href="https://github.com/GabrielGauss/irl-public-docs">Docs</a>
    <a href="/swagger-ui/" class="primary">Open Swagger UI →</a>
  </div>
</nav>

<div class="container">
  <div class="hero">
    <div class="hero-status">
      <span class="status-dot"></span>
      API · irl.macropulse.live · TLS · PostgreSQL
    </div>
    <h1>IRL Engine<br><span>Developer Sandbox</span></h1>
    <p class="hero-sub">
      Cryptographic pre-execution compliance gateway for autonomous trading agents.
      Every authorize call seals the agent's reasoning with SHA-256 before the order reaches the exchange.
      Three demo agents are pre-seeded — no registration required to try the API.
    </p>
    <div class="hero-actions">
      <a href="/swagger-ui/" class="btn-primary">Interactive API →</a>
      <a href="/openapi.json" class="btn-ghost">OpenAPI JSON</a>
      <a href="/irl/health" class="btn-ghost">Health</a>
    </div>
  </div>
</div>

<div class="container">
  <div class="section">
    <div class="section-label">Quick Start</div>
    <h2>Three calls, full audit chain</h2>
    <p class="section-sub">Use a demo agent_id below. No API key required for the public sandbox.</p>
    <div class="steps">
      <div class="step">
        <div class="step-num">1</div>
        <div class="step-body">
          <div class="step-title">Authorize — seal the reasoning before the order</div>
          <div class="step-desc">Returns <span class="mono" style="font-size:0.78rem;color:var(--text)">reasoning_hash</span> + <span class="mono" style="font-size:0.78rem;color:var(--text)">trace_id</span>. Embed both in the exchange order metadata.</div>
<pre><span class="comment"># POST /irl/authorize</span>
<span class="comment"># Header: Authorization: Bearer &lt;token&gt;</span>
{
  <span class="key">"trace_id"</span>: <span class="str">"&lt;uuid-v4&gt;"</span>,
  <span class="key">"agent_id"</span>: <span class="str">"00000000-0000-4000-a000-000000000001"</span>,
  <span class="key">"action"</span>: { <span class="key">"Long"</span>: <span class="val">1.5</span> },
  <span class="key">"asset"</span>: <span class="str">"BTC/USD"</span>,
  <span class="key">"order_type"</span>: <span class="str">"Market"</span>,
  <span class="key">"quantity"</span>: <span class="val">1.5</span>,
  <span class="key">"client_order_id"</span>: <span class="str">"ord-001"</span>,
  <span class="key">"heartbeat_seq"</span>: <span class="val">1</span>,
  <span class="key">"policy_id"</span>: <span class="str">"default"</span>,
  <span class="key">"policy_version"</span>: <span class="str">"1.0"</span>
}
<span class="comment"># → { "reasoning_hash": "sha256...", "trace_id": "...", "policy_result": "ALLOWED" }</span></pre>
        </div>
      </div>
      <div class="step">
        <div class="step-num">2</div>
        <div class="step-body">
          <div class="step-title">Execute — submit to exchange, get tx_id back</div>
          <div class="step-desc">The agent sends the order. The exchange returns an execution report. IRL is not in this path — the agent acts autonomously.</div>
        </div>
      </div>
      <div class="step">
        <div class="step-num">3</div>
        <div class="step-body">
          <div class="step-title">Bind — close the chain with final_proof</div>
          <div class="step-desc">IRL computes <span class="mono" style="font-size:0.78rem;color:var(--text)">final_proof = SHA-256(reasoning_hash ‖ exchange_tx_id)</span>. Chain closed — verifiable by any auditor.</div>
<pre><span class="comment"># POST /irl/bind-execution</span>
{
  <span class="key">"trace_id"</span>: <span class="str">"&lt;same uuid from step 1&gt;"</span>,
  <span class="key">"exchange_tx_id"</span>: <span class="str">"exch-abc-123"</span>,
  <span class="key">"execution_status"</span>: <span class="str">"FILLED"</span>,
  <span class="key">"execution_price"</span>: <span class="val">43250.00</span>
}
<span class="comment"># → { "final_proof": "sha256...", "verification_status": "MATCHED" }</span></pre>
        </div>
      </div>
    </div>
  </div>
</div>

<div class="container">
  <div class="section">
    <div class="section-label">Endpoints</div>
    <h2>API Reference</h2>
    <p class="section-sub">Full schema + try-it-out in Swagger UI. OpenAPI 3.1 JSON for Postman / SDK generators.</p>
    <div class="cards">
      <a class="card" href="/swagger-ui/">
        <div class="card-method">Interactive</div>
        <h3>Swagger UI</h3>
        <p>Try all 14 endpoints in-browser. Authorize with demo token. Full request/response schema.</p>
        <div class="card-path">/swagger-ui/</div>
      </a>
      <a class="card" href="/openapi.json">
        <div class="card-method">GET</div>
        <h3>OpenAPI 3.1 Spec</h3>
        <p>Machine-readable JSON. Import into Postman, Insomnia, or generate a typed client.</p>
        <div class="card-path">/openapi.json</div>
      </a>
      <a class="card" href="/irl/health">
        <div class="card-method">GET</div>
        <h3>Health Check</h3>
        <p>Liveness probe. No auth required. Returns <span class="mono" style="font-size:0.75rem">{"status":"ok"}</span>.</p>
        <div class="card-path">/irl/health</div>
      </a>
      <a class="card" href="https://macropulse.live/irl" target="_blank" rel="noopener">
        <div class="card-method">Overview</div>
        <h3>IRL Overview &amp; Pricing</h3>
        <p>Product overview, editions, pricing, and regulatory mapping.</p>
        <div class="card-path">macropulse.live/irl</div>
      </a>
      <a class="card" href="https://macropulse.live/irl-whitepaper" target="_blank" rel="noopener">
        <div class="card-method">Whitepaper</div>
        <h3>Protocol Specification</h3>
        <p>Full cryptographic design, bitemporal model, audit chain, regulatory mapping.</p>
        <div class="card-path">macropulse.live/irl-whitepaper</div>
      </a>
      <a class="card" href="https://github.com/GabrielGauss/irl-public-docs" target="_blank" rel="noopener">
        <div class="card-method">GitHub</div>
        <h3>Public Docs</h3>
        <p>Developer guide, SDK reference, exchange integration, compliance guide, SLA.</p>
        <div class="card-path">github.com/GabrielGauss/irl-public-docs</div>
      </a>
    </div>
  </div>
</div>

<div class="container">
  <div class="section">
    <div class="section-label">Demo Agents</div>
    <h2>Pre-seeded sandbox agents</h2>
    <p class="section-sub">Use any of these agent_ids in <span class="mono" style="font-size:0.82rem">POST /irl/authorize</span>. All are Active, max_notional 10,000, allowed regimes 0–2.</p>
    <div class="agents-wrap">
      <table class="agents-table">
        <thead><tr><th>Use case</th><th>agent_id</th><th>Status</th></tr></thead>
        <tbody>
          <tr>
            <td>Crypto</td>
            <td>00000000-0000-4000-a000-000000000001</td>
            <td><span class="tag-active">Active</span></td>
          </tr>
          <tr>
            <td>Equities</td>
            <td>00000000-0000-4000-a000-000000000002</td>
            <td><span class="tag-active">Active</span></td>
          </tr>
          <tr>
            <td>Futures</td>
            <td>00000000-0000-4000-a000-000000000003</td>
            <td><span class="tag-active">Active</span></td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</div>

<footer>
  <span class="footer-brand">IRL Engine · MacroPulse Research · irl.macropulse.live</span>
  <div class="footer-links">
    <a href="https://macropulse.live/irl">IRL Overview</a>
    <a href="https://macropulse.live/irl-whitepaper">Whitepaper</a>
    <a href="https://github.com/GabrielGauss/irl-public-docs">Public Docs</a>
    <a href="https://macropulse.live">MacroPulse</a>
  </div>
</footer>

</body>
</html>
"#;

/// GET /irl/trace/:trace_id
///
/// Returns the full Reasoning_Trace_v1 JSON for forensic audit replay.
/// Overlays live binding fields (final_proof, verification_status) on the stored trace.
/// Decrypts encrypted rows (encryption_version=1) transparently.
pub async fn get_trace(
    State(state): State<AppState>,
    Path(trace_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let pool = state.readonly_pool.as_ref().unwrap_or(&state.pool);
    let trace = db::get_trace_json(pool, trace_id, state.key_provider.as_deref()).await?;
    Ok(Json(trace))
}

#[derive(Deserialize)]
pub struct PendingQuery {
    /// Minimum age in seconds before a trace appears in the pending list.
    /// Default: 0 (show all PENDING traces).
    pub age_seconds: Option<i64>,
}

/// GET /irl/pending
///
/// Returns PENDING traces older than `age_seconds` (default: all PENDING).
/// Used by operators to identify unconfirmed intents awaiting bind-execution.
/// Decrypts encrypted rows (encryption_version=1) transparently.
pub async fn get_pending(
    State(state): State<AppState>,
    Query(q): Query<PendingQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let age = q.age_seconds.unwrap_or(0);
    let pool = state.readonly_pool.as_ref().unwrap_or(&state.pool);
    let traces = db::get_pending_traces(pool, age, state.key_provider.as_deref()).await?;
    Ok(Json(
        serde_json::json!({ "count": traces.len(), "traces": traces }),
    ))
}

/// GET /irl/orphans
///
/// Returns EXPIRED and DIVERGENT traces — trades that were either never confirmed
/// or where the exchange execution differed from the authorized intent.
/// Decrypts encrypted rows (encryption_version=1) transparently.
pub async fn get_orphans(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let pool = state.readonly_pool.as_ref().unwrap_or(&state.pool);
    let traces = db::get_orphan_traces(pool, state.key_provider.as_deref()).await?;
    Ok(Json(
        serde_json::json!({ "count": traces.len(), "traces": traces }),
    ))
}

/// GET /irl/shadow-violations
///
/// Returns traces where SHADOW_MODE intercepted a policy violation.
/// Used by compliance teams to tune policies before switching to enforcement.
/// Only populated when `SHADOW_MODE=true` has been active.
/// Decrypts encrypted rows (encryption_version=1) transparently.
pub async fn get_shadow_violations(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let pool = state.readonly_pool.as_ref().unwrap_or(&state.pool);
    let traces = db::get_shadow_violations(pool, state.key_provider.as_deref()).await?;
    Ok(Json(
        serde_json::json!({ "count": traces.len(), "traces": traces }),
    ))
}

/// GET /metrics
///
/// Prometheus text exposition format. Unauthenticated by design — restrict at
/// the network layer (firewall / ingress allowlist) in production.
pub async fn metrics_handler() -> impl IntoResponse {
    match metrics::render() {
        Ok(body) => (
            StatusCode::OK,
            [("content-type", "text/plain; version=0.0.4; charset=utf-8")],
            body,
        )
            .into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

/// GET /irl/health
///
/// Returns `{"status": "ok"}` with HTTP 200.
/// When MTLS_ENABLED=true, also includes `cert_expiry_status`.
/// Intended for load-balancer and container health probes.
pub async fn health(State(state): State<AppState>) -> Json<serde_json::Value> {
    use crate::tls::expiry::{check_cert_expiry, CertExpiryStatus};

    if state.config.mtls_enabled {
        if let Some(not_after) = state.cert_expiry_not_after {
            let expiry_status = match check_cert_expiry(not_after) {
                CertExpiryStatus::Ok => serde_json::json!("ok"),
                CertExpiryStatus::ExpiringSoon { days_remaining } => {
                    serde_json::json!({ "warning": format!("expires in {days_remaining} day(s)") })
                }
                CertExpiryStatus::Expired => serde_json::json!("expired"),
            };
            return Json(serde_json::json!({
                "status": "ok",
                "cert_expiry_status": expiry_status
            }));
        }
    }
    Json(serde_json::json!({ "status": "ok" }))
}
