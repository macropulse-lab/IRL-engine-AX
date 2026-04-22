pub mod asset;
pub mod audit;
pub mod auth;
pub mod backfill;
pub mod binding;
pub mod config;
pub mod db;
pub mod encryption;
pub mod errors;
pub mod gdpr;
pub mod heartbeat;
pub mod kms;
pub mod merkle;
pub mod metrics;
pub mod middleware;
pub mod mta;
pub mod openapi;
pub mod policy;
pub mod rate_limit;
pub mod registry;
pub mod routes;
pub mod seal;
pub mod shadow_mode;
pub mod snapshot;
pub mod time;
pub mod tls;
pub mod token_manager;
pub mod verifier;

use std::sync::Arc;
pub use token_manager::TokenManager;

use auth::build_auth_state;
use axum::{
    extract::DefaultBodyLimit,
    middleware as axum_middleware,
    routing::{delete, get, patch, post},
    Extension, Router,
};
use heartbeat::HeartbeatValidator;
use middleware::client_cert::{client_cert_middleware, PeerCertDer};
use mta::MtaClient;
use rate_limit::RateLimiter;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

/// Shared application state injected into all route handlers.
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<config::Config>,
    pub pool: sqlx::PgPool,
    /// DB-02: optional read-replica pool for analytics SELECT routes.
    /// When DB_READONLY_URL is set, analytics GET routes use this pool.
    /// Falls back to primary `pool` when None.
    pub readonly_pool: Option<sqlx::PgPool>,
    pub heartbeat_validator: Arc<HeartbeatValidator>,
    /// The active Market Truth Anchor client.
    /// Any type implementing MtaClient can be substituted here without
    /// touching route handlers or the policy engine.
    pub mta_client: Arc<dyn MtaClient>,
    /// KMS envelope key provider. None = plaintext mode (encryption_version=0).
    pub key_provider: Option<Arc<dyn kms::KeyProvider>>,
    /// DB-backed shadow mode cache. Hot-path reads are O(1) AtomicBool reads.
    /// Background refresh keeps it in sync with `irl.system_config` every 30 s.
    pub shadow_mode: Arc<shadow_mode::ShadowModeCache>,
    /// DB-backed token manager. Used by admin token endpoints to issue/revoke
    /// tokens and refresh the in-memory cache immediately.
    pub token_manager: Arc<TokenManager>,
    /// Server TLS certificate expiry time. Populated by main.rs when TLS is active.
    /// Used by the health endpoint to surface cert_expiry_status.
    pub cert_expiry_not_after: Option<std::time::SystemTime>,
}

/// Build the application router from a fully initialised `AppState`.
///
/// Separating router construction from `main` lets integration tests build
/// the same app against a real database without starting a TCP listener.
pub fn build_router(state: AppState) -> Router {
    let rate_limiter = RateLimiter::new(state.config.rate_limit_per_second);
    let auth_state = build_auth_state(state.token_manager.clone(), rate_limiter);
    let max_body = state.config.max_body_bytes;

    let protected = Router::new()
        .route("/irl/authorize", post(routes::authorize::authorize))
        .route("/irl/bind-execution", post(routes::bind::bind_execution))
        .route("/irl/trace/:trace_id", get(routes::get_trace))
        .route("/irl/pending", get(routes::get_pending))
        .route("/irl/orphans", get(routes::get_orphans))
        .route("/irl/agents", post(routes::agents::register_agent))
        .route("/irl/agents", get(routes::agents::list_agents))
        .route("/irl/agents/:id", get(routes::agents::get_agent))
        .route(
            "/irl/agents/:id/status",
            patch(routes::agents::update_agent_status),
        )
        .route("/irl/shadow-violations", get(routes::get_shadow_violations))
        .route("/irl/traces", get(routes::traces::list_traces))
        .layer(axum_middleware::from_fn_with_state(
            auth_state.clone(),
            auth::require_bearer,
        ));

    // Owner-only admin routes — require_owner runs after require_bearer.
    // Layers are applied bottom-up, so require_bearer is the outer layer.
    let admin_routes = Router::new()
        .route(
            "/irl/admin/shadow-mode",
            get(routes::admin::shadow_mode_get),
        )
        .route(
            "/irl/admin/shadow-mode",
            post(routes::admin::shadow_mode_set),
        )
        .route("/irl/admin/audit-log", get(routes::admin::audit_log_query))
        .route(
            "/irl/admin/gdpr-erase/:agent_id",
            post(routes::admin::gdpr_erase_handler),
        )
        .route("/irl/admin/tokens", post(routes::tokens::token_issue))
        .route(
            "/irl/admin/tokens/:token_id",
            delete(routes::tokens::token_revoke),
        )
        .layer(axum_middleware::from_fn_with_state(
            auth_state.clone(),
            auth::require_owner,
        ))
        .layer(axum_middleware::from_fn_with_state(
            auth_state,
            auth::require_bearer,
        ));

    Router::new()
        .route("/", get(routes::landing))
        .route("/irl/health", get(routes::health))
        .route("/metrics", get(routes::metrics_handler))
        // OpenAPI spec + Swagger UI (no auth required — useful for dev/demo instances)
        // SwaggerUi serves /openapi.json automatically via .url(...)
        .merge(SwaggerUi::new("/swagger-ui").url("/openapi.json", openapi::ApiDoc::openapi()))
        .merge(protected)
        .merge(admin_routes)
        .layer(axum_middleware::from_fn(client_cert_middleware))
        .layer(Extension(PeerCertDer::default()))
        .layer(DefaultBodyLimit::max(max_body))
        .with_state(state)
}
