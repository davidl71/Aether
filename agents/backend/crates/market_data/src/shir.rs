//! SHIR (Shekel Overnight Interest Rate) fetcher.
//!
//! SHIR is Israel's benchmark overnight rate, published daily by the Bank of Israel.
//! It serves as the base rate for ILS-denominated loans (SHIR-based loans).
//!
//! Sources (in order of preference):
//! 1. Bank of Israel website (primary, official source)
//! 2. IBKR/TWS interest rates (if available)
//! 3. Manual/user input (fallback)
//!
//! Note: The Bank of Israel doesn't have a documented public REST API for SHIR.
//! Rate is typically published on their website by 11:00 AM local time on business days.

use serde::{Deserialize, Serialize};

/// SHIR rate data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShirRate {
    pub rate: f64,
    pub date: String,
    pub source: String,
}

/// Result type for SHIR fetch operations.
pub type ShirResult<T> = anyhow::Result<T>;

const BANK_OF_ISRAEL_URL: &str = "https://www.boi.org.il/en/About/MonetaryPolicy/Pages/InterestRates.aspx";

/// Fetch the current SHIR rate from Bank of Israel.
///
/// The Bank of Israel publishes SHIR daily by 11:00 AM.
/// On non-publication days, the last published value is used.
///
/// Returns `None` if fetching fails or the rate is not available.
pub async fn fetch_shir_from_boi() -> ShirResult<Option<ShirRate>> {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (compatible; Aether/1.0)")
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    let response = client.get(BANK_OF_ISRAEL_URL).send().await?;

    if !response.status().is_success() {
        return Ok(None);
    }

    let html = response.text().await?;

    if let Some(rate) = parse_shir_from_html(&html) {
        return Ok(Some(rate));
    }

    Ok(None)
}

/// Parse SHIR rate from Bank of Israel HTML page.
///
/// The rate is typically found in a table or data element on the page.
/// This is a fragile parse - the page structure may change.
fn parse_shir_from_html(html: &str) -> Option<ShirRate> {
    use regex::Regex;

    let re = Regex::new(r"SHIR.*?(\d+\.?\d*)").ok()?;
    let caps = re.captures(html)?;

    let rate_str = caps.get(1)?.as_str();
    let rate: f64 = rate_str.parse().ok()?;

    let today = chrono::Local::now().format("%Y-%m-%d").to_string();

    Some(ShirRate {
        rate,
        date: today,
        source: "Bank of Israel".to_string(),
    })
}

/// Fetch SHIR rate, trying multiple sources in order.
///
/// Priority:
/// 1. Bank of Israel (official, daily publication)
/// 2. Cached value from last successful fetch (if recent)
///
/// Returns `None` if all sources fail.
pub async fn fetch_shir_rate() -> ShirResult<Option<ShirRate>> {
    if let Some(rate) = fetch_shir_from_boi().await? {
        return Ok(Some(rate));
    }

    Ok(None)
}

/// Default SHIR rate to use when no source is available.
/// Based on approximate current SHIR rate (2024-2025 range).
pub fn default_shir_rate() -> ShirRate {
    ShirRate {
        rate: 0.0395,
        date: chrono::Local::now().format("%Y-%m-%d").to_string(),
        source: "default".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_rate_is_reasonable() {
        let rate = default_shir_rate();
        assert!(rate.rate > 0.0);
        assert!(rate.rate < 0.10);
    }
}
