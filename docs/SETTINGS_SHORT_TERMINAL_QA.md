# Settings tab — short-terminal QA

Manual matrix after layout changes in `tui_service` (`ui/settings.rs`, `ui/settings_health.rs`). Run `cargo run -p tui_service`, open **Settings**, then confirm panels stay usable (no blank core, borders visible, text truncates instead of wrapping badly).

## Heights (full-width Settings tab)

Use a fixed width (e.g. 120 cols); vary **rows**:

| Rows | Expect |
|------|--------|
| 24+ | Wide layout if width ≥ 110 and height ≥ 18: health+config top row; symbols/sources/alpaca bottom. |
| 17 | Stacked layout: health band + four middle sections + hint row. |
| 12 | Stacked: health shrinks (≤7 lines); mid sections share remaining rows evenly. |
| 10 | Stacked: health ≤5 lines; **System health** flow + **Components** still partition height (no over-constraint). |
| 8 | Extreme: mid quarters may be 1–2 rows each; scroll/clipping acceptable. |

## Widths (stacked / narrow)

| Width | Expect |
|-------|--------|
| ≥110, h≥18 | Wide layout on full tab. |
| 92–109 | Stacked; embedded Operations column uses `settings_layout_embedded` threshold 92. |
| &lt;92 | Stacked; truncation helpers apply in section bodies. |

## Spot checks

- [ ] **System health**: Transport / Services tree visible; **Components** strip has non-zero height when health area ≥ 2 rows.
- [ ] **Config overrides**, **Watchlist symbols**, **Data sources**, **Alpaca** each show a titled block in stacked mode.
- [ ] Hint row (bottom) always one line.
- [ ] Operations workspace: Settings in right column at typical laptop size still renders all sections.

## Related

- Pane model: `docs/TUI_PANE_MODEL.md` §7.
