// build.rs - This intentionally fails to remind developers to use agents/backend/

fn main() {
    // Print a very visible warning
    println!("cargo:warning=⚠️  ╔════════════════════════════════════════════════════════════╗");
    println!("cargo:warning=⚠️  ║  WRONG DIRECTORY: You are building from project root       ║");
    println!("cargo:warning=⚠️  ╠════════════════════════════════════════════════════════════╣");
    println!("cargo:warning=⚠️  ║                                                            ║");
    println!("cargo:warning=⚠️  ║  The Rust workspace is at: agents/backend/                 ║");
    println!("cargo:warning=⚠️  ║                                                            ║");
    println!("cargo:warning=⚠️  ║  Run cargo commands from there:                            ║");
    println!("cargo:warning=⚠️  ║                                                            ║");
    println!("cargo:warning=⚠️  ║      cd agents/backend && cargo build                      ║");
    println!("cargo:warning=⚠️  ║      cd agents/backend && cargo test                       ║");
    println!("cargo:warning=⚠️  ║      cd agents/backend && cargo run -p backend_service     ║");
    println!("cargo:warning=⚠️  ║                                                            ║");
    println!("cargo:warning=⚠️  ╚════════════════════════════════════════════════════════════╝");

    // Fail the build to force the developer to notice
    panic!("\n\n❌ BUILD STOPPED: Please run cargo commands from agents/backend/ directory\n   cd agents/backend && cargo build\n\n");
}
