use crate::config::{self, Config, Lockfile};
use crate::lockfile;
use anyhow::Result;
use std::path::Path;

/// Format a single dependency for display
///
/// # Arguments
/// * `dep` - The dependency to format
/// * `lockfile` - The lockfile to check for synced SHA
///
/// # Returns
/// Formatted string for the dependency
pub fn format_dependency(dep: &config::Dependency, lockfile: &Lockfile) -> String {
    let rev_display = dep.rev.as_deref().unwrap_or("(HEAD)");

    let synced_info = lockfile
        .locks
        .iter()
        .find(|entry| entry.name == dep.name)
        .map(|entry| format!("[synced: {}]", &entry.sha[..7.min(entry.sha.len())]))
        .unwrap_or_else(|| "[not synced]".to_string());

    let paths_display = dep.paths.join(", ");

    format!(
        "{}  {}  {}  paths: {}  out: {}  {}",
        dep.name, dep.repo, rev_display, paths_display, dep.out, synced_info
    )
}

/// List all dependencies
///
/// # Arguments
/// * `config` - Config containing dependencies
/// * `lockfile` - Lockfile containing sync status
///
/// # Returns
/// Vector of formatted dependency lines
pub fn list_dependencies(config: &Config, lockfile: &Lockfile) -> Vec<String> {
    config
        .deps
        .iter()
        .map(|dep| format_dependency(dep, lockfile))
        .collect()
}

/// Run the ls command
pub fn run_ls(config_path: &Path, lockfile_path: &Path) -> Result<()> {
    let config = config::read_config(config_path)?;

    if config.deps.is_empty() {
        println!("No dependencies configured.");
        return Ok(());
    }

    let lockfile = lockfile::read_lockfile(lockfile_path)?;
    let lines = list_dependencies(&config, &lockfile);

    for line in lines {
        println!("{line}");
    }

    Ok(())
}

/// Run ls with default paths
pub fn run_ls_default() -> Result<()> {
    run_ls(
        Path::new(config::CONFIG_PATH),
        Path::new(config::LOCKFILE_PATH),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, Dependency, LockEntry, Lockfile};
    use tempfile::TempDir;

    #[test]
    fn test_format_dependency_with_rev_and_synced() {
        let dep = Dependency {
            name: "api".to_string(),
            repo: "https://github.com/example/api.git".to_string(),
            rev: Some("v2.0".to_string()),
            paths: vec!["proto/".to_string()],
            out: "./vendor/api".to_string(),
            hooks: vec![],
        };
        let lockfile = Lockfile {
            locks: vec![LockEntry {
                name: "api".to_string(),
                repo: "https://github.com/example/api.git".to_string(),
                rev: "v2.0".to_string(),
                sha: "abc123def456".to_string(),
            }],
        };

        let result = format_dependency(&dep, &lockfile);
        assert_eq!(
            result,
            "api  https://github.com/example/api.git  v2.0  paths: proto/  out: ./vendor/api  [synced: abc123d]"
        );
    }

    #[test]
    fn test_format_dependency_without_rev() {
        let dep = Dependency {
            name: "api".to_string(),
            repo: "https://github.com/example/api.git".to_string(),
            rev: None,
            paths: vec!["proto/".to_string()],
            out: "./vendor/api".to_string(),
            hooks: vec![],
        };
        let lockfile = Lockfile { locks: vec![] };

        let result = format_dependency(&dep, &lockfile);
        assert_eq!(
            result,
            "api  https://github.com/example/api.git  (HEAD)  paths: proto/  out: ./vendor/api  [not synced]"
        );
    }

    #[test]
    fn test_format_dependency_multiple_paths() {
        let dep = Dependency {
            name: "schemas".to_string(),
            repo: "https://github.com/example/schemas.git".to_string(),
            rev: Some("main".to_string()),
            paths: vec!["proto/".to_string(), "openapi/".to_string()],
            out: "./vendor/schemas".to_string(),
            hooks: vec![],
        };
        let lockfile = Lockfile { locks: vec![] };

        let result = format_dependency(&dep, &lockfile);
        assert_eq!(
            result,
            "schemas  https://github.com/example/schemas.git  main  paths: proto/, openapi/  out: ./vendor/schemas  [not synced]"
        );
    }

    #[test]
    fn test_list_dependencies_multiple() {
        let config = Config {
            deps: vec![
                Dependency {
                    name: "api".to_string(),
                    repo: "https://github.com/example/api.git".to_string(),
                    rev: None,
                    paths: vec!["proto/".to_string()],
                    out: "./vendor/api".to_string(),
                    hooks: vec![],
                },
                Dependency {
                    name: "ui".to_string(),
                    repo: "https://github.com/example/ui.git".to_string(),
                    rev: Some("v2.0".to_string()),
                    paths: vec!["dist/".to_string()],
                    out: "./vendor/ui".to_string(),
                    hooks: vec![],
                },
            ],
        };
        let lockfile = Lockfile {
            locks: vec![LockEntry {
                name: "api".to_string(),
                repo: "https://github.com/example/api.git".to_string(),
                rev: "HEAD".to_string(),
                sha: "abc123d".to_string(),
            }],
        };

        let lines = list_dependencies(&config, &lockfile);
        let expected = vec![
            "api  https://github.com/example/api.git  (HEAD)  paths: proto/  out: ./vendor/api  [synced: abc123d]",
            "ui  https://github.com/example/ui.git  v2.0  paths: dist/  out: ./vendor/ui  [not synced]",
        ];
        assert_eq!(lines, expected);
    }

    #[test]
    fn test_list_dependencies_empty() {
        let config = Config { deps: vec![] };
        let lockfile = Lockfile { locks: vec![] };

        let lines = list_dependencies(&config, &lockfile);
        assert!(lines.is_empty());
    }

    #[test]
    fn test_run_ls_empty_deps() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join(".skem.yaml");

        let config = Config { deps: vec![] };
        config::write_config(&config_path, &config).unwrap();

        let lockfile_path = temp_dir.path().join(".skem.lock");
        let result = run_ls(&config_path, &lockfile_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_ls_config_not_found_fails() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join(".skem.yaml");
        let lockfile_path = temp_dir.path().join(".skem.lock");

        let result = run_ls(&config_path, &lockfile_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_run_ls_without_lockfile() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join(".skem.yaml");
        let lockfile_path = temp_dir.path().join(".skem.lock");

        let config = Config {
            deps: vec![Dependency {
                name: "api".to_string(),
                repo: "https://github.com/example/api.git".to_string(),
                rev: None,
                paths: vec!["proto/".to_string()],
                out: "./vendor/api".to_string(),
                hooks: vec![],
            }],
        };
        config::write_config(&config_path, &config).unwrap();

        // No lockfile, should still succeed
        let result = run_ls(&config_path, &lockfile_path);
        assert!(result.is_ok());
    }
}
