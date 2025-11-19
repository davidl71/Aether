//! Encoding detection and conversion for Hebrew text

use crate::errors::Result;
use encoding_rs::{UTF_8, WINDOWS_1255};

/// Detect and convert file encoding to UTF-8
pub fn decode_to_utf8(bytes: &[u8]) -> Result<String> {
    // Try UTF-8 first
    if let Ok(utf8_str) = std::str::from_utf8(bytes) {
        return Ok(utf8_str.to_string());
    }

    // Try Windows-1255 (Hebrew)
    let (decoded, _, had_errors) = WINDOWS_1255.decode(bytes);
    if !had_errors {
        return Ok(decoded.to_string());
    }

    // Fallback: try UTF-8 with replacement characters
    let (decoded, _) = UTF_8.decode_with_bom_removal(bytes);
    Ok(decoded.to_string())
}

/// Get encoding name for logging
pub fn detect_encoding(bytes: &[u8]) -> &'static str {
    if std::str::from_utf8(bytes).is_ok() {
        "UTF-8"
    } else {
        "Windows-1255"
    }
}
