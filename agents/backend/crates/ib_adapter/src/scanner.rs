//! TWS scanner-based discovery (stub).
//!
//! Market scanners return lists of contracts by pre-defined or parameterized screens
//! (e.g. hot by volume, high option volume put/call ratio). Use for discovery of
//! underlyings to then request market data for.
//!
//! **Flow:** `reqScannerParameters` → `reqScannerSubscription` → `scannerData` /
//! `scannerDataEnd` → `cancelScannerSubscription`. Scanner results do **not** include
//! market data (bid/ask/last); use `reqMktData` separately if needed.
//!
//! **TWS message IDs (protobuf wire, min server version 210):**
//! - `REQ_SCANNER_PARAMETERS` — request XML of available scan codes/instruments/locations
//! - `REQ_SCANNER_SUBSCRIPTION` — start a scan (subscription + options + filter options)
//! - `CANCEL_SCANNER_SUBSCRIPTION` — stop updates for a reqId
//!
//! **Response/callbacks:** `scannerParameters(xml)`, `scannerData(reqId, rank, contractDetails, …)`,
//! `scannerDataEnd(reqId)`.
//!
//! **Limits:** Max 50 results per scan code per subscription; max 10 API scanner
//! subscriptions active at once.
//!
//! **References:**
//! - [TWS_API_LEARNINGS_FROM_SIBLING_REPO.md §6](../../../../docs/platform/TWS_API_LEARNINGS_FROM_SIBLING_REPO.md#6-scanner-subscription)
//! - [TWS API Market Scanners](https://interactivebrokers.github.io/tws-api/market_scanners.html)

/// Placeholder for TWS ScannerSubscription (C++ `ScannerSubscription.h` / Python `scanner.py`).
///
/// Key fields for building a scan: instrument (e.g. `"STK"`, `"IND.US"`),
/// locationCode (e.g. `"STK.US.MAJOR"`, `"IND.US"`), scanCode (e.g. `"HOT_BY_VOLUME"`,
/// `"HIGH_OPT_VOLUME_PUT_CALL_RATIO"`), plus optional filters (abovePrice, aboveVolume, etc.).
#[derive(Debug, Clone, Default)]
pub struct ScannerSubscription {
    /// Instrument filter, e.g. `"STK"`, `"STOCK.EU"`, `"IND.US"`.
    pub instrument: String,
    /// Location/exchange, e.g. `"STK.US.MAJOR"`, `"IND.US"`.
    pub location_code: String,
    /// Scan type, e.g. `"HOT_BY_VOLUME"`, `"MOST_ACTIVE"`, `"HIGH_OPT_VOLUME_PUT_CALL_RATIO"`.
    pub scan_code: String,
    /// Max results (-1 = default, often 50 max).
    pub number_of_rows: i32,
}

impl ScannerSubscription {
    /// Hot US stocks by volume (STK, STK.US.MAJOR, HOT_BY_VOLUME).
    pub fn hot_us_stk_by_volume() -> Self {
        Self {
            instrument: "STK".to_string(),
            location_code: "STK.US.MAJOR".to_string(),
            scan_code: "HOT_BY_VOLUME".to_string(),
            number_of_rows: -1,
        }
    }

    /// High option volume put/call ratio, US indexes (IND.US, IND.US, HIGH_OPT_VOLUME_PUT_CALL_RATIO).
    pub fn high_opt_volume_pc_ratio_us_indexes() -> Self {
        Self {
            instrument: "IND.US".to_string(),
            location_code: "IND.US".to_string(),
            scan_code: "HIGH_OPT_VOLUME_PUT_CALL_RATIO".to_string(),
            number_of_rows: -1,
        }
    }
}
