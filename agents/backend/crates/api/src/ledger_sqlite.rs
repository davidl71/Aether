//! Read-only SQLite access to `ledger.db` (shared by discount bank bank-accounts import and ledger journal).

use std::path::{Path, PathBuf};
use std::str::FromStr;

use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;

use crate::project_paths::discover_workspace_root;

/// Expand leading `~/` using `HOME`.
pub fn expand_home(path: &str) -> PathBuf {
    if let Some(stripped) = path.strip_prefix("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return Path::new(&home).join(stripped);
        }
    }
    PathBuf::from(path)
}

/// Resolve path to an existing ledger SQLite file (env + repo candidates).
pub fn ledger_database_path() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("LEDGER_DATABASE_PATH") {
        let expanded = expand_home(&path);
        if expanded.exists() {
            return Some(expanded);
        }
    }

    let repo_root = discover_workspace_root().or_else(|| {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|path| path.parent())
            .and_then(|path| path.parent())
            .map(Path::to_path_buf)
    })?;

    let candidates = [
        repo_root.join("ledger.db"),
        repo_root.join("agents/backend/ledger.db"),
        repo_root.join("agents/backend/data/ledger.db"),
        repo_root.join("data/ledger.db"),
        PathBuf::from(std::env::var("HOME").ok()?).join(".ledger/ledger.db"),
    ];

    candidates.into_iter().find(|path| path.exists())
}

/// Open a small read-only pool to the ledger database.
pub async fn open_ledger_pool() -> Result<SqlitePool, String> {
    let path = ledger_database_path().ok_or_else(|| "Ledger database not found".to_string())?;
    let uri = format!("sqlite://{}", path.display());
    let options = SqliteConnectOptions::from_str(&uri)
        .map_err(|error| format!("Invalid ledger database path: {error}"))?
        .read_only(true);
    SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .map_err(|error| format!("Failed to open ledger database: {error}"))
}
