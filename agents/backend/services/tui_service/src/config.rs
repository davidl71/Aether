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
}

impl TuiConfig {
    pub fn from_env() -> Self {
        Self {
            nats_url: std::env::var("NATS_URL")
                .unwrap_or_else(|_| "nats://localhost:4222".into()),
            backend_id: std::env::var("BACKEND_ID").unwrap_or_else(|_| "ib".into()),
            rest_url: std::env::var("REST_URL")
                .unwrap_or_else(|_| "http://localhost:8080".into()),
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
        }
    }
}
