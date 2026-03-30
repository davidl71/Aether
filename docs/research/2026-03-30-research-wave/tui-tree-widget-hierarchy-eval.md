## Evaluate `tui-tree-widget` for hierarchy views

### Summary
`tui-tree-widget` appears active (2026-01 release), and provides a tree widget + `TreeState` that matches typical hierarchy browsing needs.

### Internet references (2026)
- Crate: https://crates.io/crates/tui-tree-widget
- API docs: https://docs.rs/tui-tree-widget/latest/tui_tree_widget/
- Source/examples: https://github.com/EdJoPaTo/tui-rs-tree-widget

### Recommendation
- Create a small spike to render a static tree with keyboard navigation.
- If it feels good, map a real domain hierarchy (accounts/strategies) into `TreeItem`.
