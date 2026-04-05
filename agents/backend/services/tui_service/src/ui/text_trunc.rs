//! Char-safe string truncation for fixed-width TUI cells (Unicode-aware).
//!
//! Prefer this over byte slicing (`s[..n]`), which can panic on UTF-8 boundaries.

/// Truncate to at most `max_chars` Unicode scalar values, appending `…` when shortened.
pub fn truncate_chars(s: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }
    let n = s.chars().count();
    if n <= max_chars {
        s.to_string()
    } else {
        let t: String = s.chars().take(max_chars.saturating_sub(1)).collect();
        format!("{t}…")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_and_short_unchanged() {
        assert_eq!(truncate_chars("", 5), "");
        assert_eq!(truncate_chars("hi", 5), "hi");
    }

    #[test]
    fn long_ascii_truncates() {
        assert_eq!(truncate_chars("abcdef", 4), "abc…");
    }

    #[test]
    fn unicode_counts_chars_not_bytes() {
        let s = "日本語テスト"; // 5 chars
        assert_eq!(truncate_chars(s, 4), "日本語…");
    }
}
