use std::path::{Path, PathBuf};

const PROJECT_ROOT_OVERRIDE_ENV: &str = "IB_BOX_SPREAD_PROJECT_ROOT";
const SHARED_CONFIG_ENV: &str = "IB_BOX_SPREAD_CONFIG";

pub fn discover_workspace_root() -> Option<PathBuf> {
    if let Ok(explicit) = std::env::var(PROJECT_ROOT_OVERRIDE_ENV) {
        let explicit = expand_home_path(PathBuf::from(explicit));
        if is_workspace_root(&explicit) {
            return Some(explicit);
        }
    }

    for start in workspace_root_search_roots() {
        if let Some(root) = find_workspace_root_from(&start) {
            return Some(root);
        }
    }

    None
}

pub fn shared_config_candidate_paths() -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    let mut add = |path: PathBuf| {
        if !candidates.iter().any(|existing| existing == &path) {
            candidates.push(path);
        }
    };

    if let Ok(explicit) = std::env::var(SHARED_CONFIG_ENV) {
        let explicit = expand_home_path(PathBuf::from(explicit));
        if explicit.is_absolute() {
            add(explicit);
        } else if let Ok(cwd) = std::env::current_dir() {
            add(cwd.join(&explicit));
            add(explicit);
        } else {
            add(explicit);
        }
    }

    if let Some(home) = std::env::var_os("HOME") {
        let home = PathBuf::from(home);
        add(home.join(".config/ib_box_spread/config.json"));
        if cfg!(target_os = "macos") {
            add(home.join("Library/Application Support/ib_box_spread/config.json"));
        }
    }

    add(PathBuf::from("/usr/local/etc/ib_box_spread/config.json"));
    add(PathBuf::from("/etc/ib_box_spread/config.json"));

    if let Some(project_root) = discover_workspace_root() {
        add(project_root.join("config/config.json"));
        add(project_root.join("config/config.example.json"));
    }

    add(PathBuf::from("config/config.json"));
    add(PathBuf::from("config/config.example.json"));

    candidates
}

fn workspace_root_search_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();

    if let Ok(cwd) = std::env::current_dir() {
        roots.push(cwd);
    }

    if let Ok(exe) = std::env::current_exe() {
        roots.push(exe);
    }

    roots.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")));
    roots
}

fn find_workspace_root_from(start: &Path) -> Option<PathBuf> {
    start
        .ancestors()
        .find(|path| is_workspace_root(path))
        .map(Path::to_path_buf)
}

fn is_workspace_root(path: &Path) -> bool {
    path.join("agents/backend/Cargo.toml").is_file() && path.join("config/schema.json").is_file()
}

fn expand_home_path(path: PathBuf) -> PathBuf {
    let text = path.to_string_lossy();
    if text == "~" {
        return std::env::var_os("HOME").map(PathBuf::from).unwrap_or(path);
    }
    if let Some(stripped) = text.strip_prefix("~/") {
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(home).join(stripped);
        }
    }
    path
}

#[cfg(test)]
mod tests {
    use std::{env, fs};

    use uuid::Uuid;

    use super::{discover_workspace_root, shared_config_candidate_paths};

    #[test]
    fn discovers_repository_root() {
        let root = discover_workspace_root().expect("workspace root");

        assert!(root.join("agents/backend/Cargo.toml").is_file());
        assert!(root.join("config/schema.json").is_file());
    }

    #[test]
    fn shared_config_candidates_include_workspace_example_config() {
        let root = discover_workspace_root().expect("workspace root");
        let expected = root.join("config/config.example.json");

        assert!(shared_config_candidate_paths().contains(&expected));
    }

    #[test]
    fn project_root_override_env_is_honored() {
        let temp_root = env::temp_dir().join(format!("ib-box-root-{}", Uuid::new_v4()));
        let cargo_toml = temp_root.join("agents/backend/Cargo.toml");
        let schema = temp_root.join("config/schema.json");

        fs::create_dir_all(cargo_toml.parent().expect("cargo parent")).expect("create cargo dir");
        fs::create_dir_all(schema.parent().expect("schema parent")).expect("create schema dir");
        fs::write(&cargo_toml, "[workspace]\n").expect("write cargo marker");
        fs::write(&schema, "{}\n").expect("write schema marker");

        env::set_var("IB_BOX_SPREAD_PROJECT_ROOT", &temp_root);
        let discovered = discover_workspace_root();
        env::remove_var("IB_BOX_SPREAD_PROJECT_ROOT");

        assert_eq!(discovered.as_deref(), Some(temp_root.as_path()));

        let _ = fs::remove_dir_all(temp_root);
    }
}
