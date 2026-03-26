//! Aether - Rust Workspace Redirect
//!
//! This binary exists to provide a helpful error message when
//! cargo commands are run from the project root instead of
//! the actual workspace at agents/backend/

fn main() {
    eprintln!(
        r#"
╔════════════════════════════════════════════════════════════════╗
║  ERROR: Running cargo from project root                        ║
╠════════════════════════════════════════════════════════════════╣
║                                                                ║
║  The Rust workspace is NOT located at the project root.        ║
║                                                                ║
║  Navigate to the actual workspace:                             ║
║                                                                ║
║      cd agents/backend                                         ║
║                                                                ║
║  Then run your cargo commands:                                 ║
║                                                                ║
║      cargo build                                               ║
║      cargo test                                                ║
║      cargo run -p backend_service                              ║
║      cargo run -p tui_service                                  ║
║      cargo run -p cli                                          ║
║                                                                ║
║  Or use the justfile shortcuts from project root:              ║
║                                                                ║
║      just build-rust                                           ║
║      just test                                                 ║
║                                                                ║
╚════════════════════════════════════════════════════════════════╝
"#
    );
    std::process::exit(1);
}
