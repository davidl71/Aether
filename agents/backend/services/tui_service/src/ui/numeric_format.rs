//! Helpers for numeric column alignment in ratatui tables.
//!
//! ## Display width caveat
//! [`display_width_cells`] counts Unicode scalar values ([`char`]s), not terminal
//! column width. Wide East Asian characters, emoji, and some grapheme clusters
//! can occupy more than one cell on screen; combining characters may occupy
//! zero. Replace with `unicode-width` or similar if you need accurate layout.

/// Approximate “display width” as a count of Unicode scalar values.
///
/// See the [module-level caveat](self#display-width-caveat) for terminal accuracy limits.
pub fn display_width_cells(s: &str) -> usize {
    s.chars().count()
}

/// Maximum [`display_width_cells`] over sample strings (0 if the iterator is empty).
pub fn max_display_width<'a, I>(samples: I) -> usize
where
    I: Iterator<Item = &'a str>,
{
    samples.map(display_width_cells).max().unwrap_or(0)
}

/// Left-pad `s` with ASCII spaces until [`display_width_cells`] is at least `width`.
///
/// If `s` is already wider than or equal to `width`, returns `s` unchanged (no truncation).
pub fn pad_left(width: usize, s: &str) -> String {
    let w = display_width_cells(s);
    if w >= width {
        return s.to_string();
    }
    let pad = width - w;
    format!("{}{}", " ".repeat(pad), s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pad_left_pads_to_width() {
        assert_eq!(pad_left(5, "ab"), "   ab");
        assert_eq!(pad_left(4, "xyz"), " xyz");
    }

    #[test]
    fn pad_left_no_pad_when_wide_enough() {
        assert_eq!(pad_left(3, "hello"), "hello");
        assert_eq!(pad_left(0, ""), "");
    }

    #[test]
    fn max_display_width_empty_is_zero() {
        assert_eq!(max_display_width(std::iter::empty()), 0);
    }

    #[test]
    fn max_display_width_takes_max_char_count() {
        let v = ["a", "bbb", "cc"];
        assert_eq!(max_display_width(v.iter().copied()), 3);
    }
}
