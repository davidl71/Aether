//! TWS-related env parsing shared by tws_market_data and tws_positions.
//!
//! TWS_CLIENT_ID is used as the base client ID; components add offsets (0, +1, etc.).
//! If the var is set but not a valid i32, we log a warning and fall back to the default
//! so operators see why their value was "ignored".

use tracing::warn;

/// Parse `TWS_CLIENT_ID` env var; use `default` if unset or invalid.
/// Logs a warning when the var is set but parsing fails (e.g. empty, non-numeric, or overflow).
pub fn parse_tws_client_id(default: i32) -> i32 {
    match std::env::var("TWS_CLIENT_ID") {
        Ok(s) => match s.trim().parse::<i32>() {
            Ok(n) => n,
            Err(_) => {
                warn!(
                    value = %s,
                    default = %default,
                    "TWS_CLIENT_ID is set but not a valid i32; using default (use an integer to avoid this)"
                );
                default
            }
        },
        Err(_) => default,
    }
}
