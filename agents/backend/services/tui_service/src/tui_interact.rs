//! tui_interact — optional ratatui-interact integration shim.
//!
//! This module is compiled only when the `tui-interact` feature is enabled.
//! Keep all direct `ratatui_interact::*` usage isolated here so the rest of the
//! TUI remains dependency-light and can evolve independently.

pub use ratatui_interact;

/// Returns true when `ratatui-interact` integration is compiled in.
pub const fn enabled() -> bool {
    true
}

/// Small prelude for local experiments (focus manager, click regions, components).
///
/// Intentionally re-exported as a single entrypoint so callsites can do:
/// `use crate::tui_interact::prelude::*;` without importing the crate directly.
pub mod prelude {
    pub use ratatui_interact::components::*;
    pub use ratatui_interact::events::*;
    pub use ratatui_interact::prelude::*;
    pub use ratatui_interact::state::*;
    pub use ratatui_interact::traits::*;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enabled_is_true() {
        assert!(enabled());
    }

    #[test]
    fn focus_manager_compiles() {
        #[derive(Clone, PartialEq, Eq, Hash)]
        enum Element {
            A,
            B,
        }

        let mut fm = ratatui_interact::state::FocusManager::new();
        fm.register(Element::A);
        fm.register(Element::B);
        fm.next();
        fm.prev();
    }
}

