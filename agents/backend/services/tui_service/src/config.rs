//! TUI configuration loaded from environment variables.

#[derive(Debug, Clone)]
pub struct TuiConfig {
    /// NATS server URL (env: NATS_URL)
    pub nats_url: String,
    /// Backend identifier used as the snapshot topic suffix (env: BACKEND_ID)
    pub backend_id: String,
    /// REST base URL for fallback polling (env: REST_URL)
    pub rest_url: String,
    /// Symbols to highlight in the dashboard (env: WATCHLIST, comma-separated)
    pub watchlist: Vec<String>,
    /// UI redraw interval in milliseconds (env: TICK_MS)
    pub tick_ms: u64,
    /// Poll interval for REST fallback in milliseconds (env: REST_POLL_MS)
    pub rest_poll_ms: u64,
    /// Enable REST fallback when NATS is unavailable (env: REST_FALLBACK=1)
    pub rest_fallback: bool,
}

impl TuiConfig {
    pub fn from_env() -> Self {
        Self {
            nats_url: std::env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".into()),
            backend_id: std::env::var("BACKEND_ID").unwrap_or_else(|_| "ib".into()),
            rest_url: std::env::var("REST_URL").unwrap_or_else(|_| "http://localhost:9090".into()),
            watchlist: std::env::var("WATCHLIST")
                .unwrap_or_else(|_| "SPX,XSP,NDX".into())
                .split(',')
                .map(|s| s.trim().to_uppercase())
                .filter(|s| !s.is_empty())
                .collect(),
            tick_ms: std::env::var("TICK_MS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(250),
            rest_poll_ms: std::env::var("REST_POLL_MS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(2000),
            rest_fallback: std::env::var("REST_FALLBACK")
                .map(|v| v == "1" || v.to_lowercase() == "true")
                .unwrap_or(true),
        }
    }
}
