use crate::config;
use crate::lockfile;
use anyhow::Result;
use std::path::Path;

/// Remove a dependency from .skem.yaml and .skem.lock
///
/// # Arguments
/// * `config_path` - Path to the config file
/// * `lockfile_path` - Path to the lockfile
/// * `name` - Name of the dependency to remove
pub fn run_rm(config_path: &Path, lockfile_path: &Path, name: &str) -> Result<()> {
    let mut config = config::read_config(config_path)?;

    let original_len = config.deps.len();
    config.deps.retain(|d| d.name != name);

    if config.deps.len() == original_len {
        anyhow::bail!("Dependency '{name}' not found in the configuration.");
    }

    config::write_config(config_path, &config)?;

    // Remove from lockfile if it exists
    if lockfile_path.exists() {
        let current_lockfile = lockfile::read_lockfile(lockfile_path)?;
        let updated_lockfile = lockfile::remove_lockfile_entry(&current_lockfile, name);
        lockfile::write_lockfile(lockfile_path, &updated_lockfile)?;
    }

    println!("Removed dependency '{name}'.");
    Ok(())
}

/// Run rm with default paths
pub fn run_rm_default(name: &str) -> Result<()> {
    run_rm(
        Path::new(config::CONFIG_PATH),
        Path::new(config::LOCKFILE_PATH),
        name,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, Dependency, LockEntry, Lockfile};
    use tempfile::TempDir;

    #[test]
    fn test_run_rm_removes_dependency() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join(".skem.yaml");
        let lockfile_path = temp_dir.path().join(".skem.lock");

        let config = Config {
            deps: vec![
                Dependency {
                    name: "dep1".to_string(),
                    repo: "https://github.com/example/dep1.git".to_string(),
                    rev: Some("main".to_string()),
                    paths: vec!["src/".to_string()],
                    out: "./vendor/dep1".to_string(),
                    hooks: vec![],
                },
                Dependency {
                    name: "dep2".to_string(),
                    repo: "https://github.com/example/dep2.git".to_string(),
                    rev: None,
                    paths: vec!["proto/".to_string()],
                    out: "./vendor/dep2".to_string(),
                    hooks: vec![],
                },
            ],
            post_hooks: vec![],
        };
        config::write_config(&config_path, &config).unwrap();

        run_rm(&config_path, &lockfile_path, "dep1").unwrap();

        let updated = config::read_config(&config_path).unwrap();
        let expected = Config {
            deps: vec![Dependency {
                name: "dep2".to_string(),
                repo: "https://github.com/example/dep2.git".to_string(),
                rev: None,
                paths: vec!["proto/".to_string()],
                out: "./vendor/dep2".to_string(),
                hooks: vec![],
            }],
            post_hooks: vec![],
        };
        assert_eq!(updated, expected);
    }

    #[test]
    fn test_run_rm_removes_from_lockfile() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join(".skem.yaml");
        let lockfile_path = temp_dir.path().join(".skem.lock");

        let config = Config {
            deps: vec![Dependency {
                name: "dep1".to_string(),
                repo: "https://github.com/example/dep1.git".to_string(),
                rev: Some("main".to_string()),
                paths: vec!["src/".to_string()],
                out: "./vendor/dep1".to_string(),
                hooks: vec![],
            }],
            post_hooks: vec![],
        };
        config::write_config(&config_path, &config).unwrap();

        let lf = Lockfile {
            locks: vec![
                LockEntry {
                    name: "dep1".to_string(),
                    repo: "https://github.com/example/dep1.git".to_string(),
                    rev: "main".to_string(),
                    sha: "abc123".to_string(),
                },
                LockEntry {
                    name: "dep2".to_string(),
                    repo: "https://github.com/example/dep2.git".to_string(),
                    rev: "v1.0".to_string(),
                    sha: "def456".to_string(),
                },
            ],
        };
        lockfile::write_lockfile(&lockfile_path, &lf).unwrap();

        run_rm(&config_path, &lockfile_path, "dep1").unwrap();

        let updated_lf = lockfile::read_lockfile(&lockfile_path).unwrap();
        let expected_lf = Lockfile {
            locks: vec![LockEntry {
                name: "dep2".to_string(),
                repo: "https://github.com/example/dep2.git".to_string(),
                rev: "v1.0".to_string(),
                sha: "def456".to_string(),
            }],
        };
        assert_eq!(updated_lf, expected_lf);
    }

    #[test]
    fn test_run_rm_nonexistent_dependency_fails() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join(".skem.yaml");
        let lockfile_path = temp_dir.path().join(".skem.lock");

        let config = Config {
            deps: vec![Dependency {
                name: "dep1".to_string(),
                repo: "https://github.com/example/dep1.git".to_string(),
                rev: None,
                paths: vec!["src/".to_string()],
                out: "./vendor/dep1".to_string(),
                hooks: vec![],
            }],
            post_hooks: vec![],
        };
        config::write_config(&config_path, &config).unwrap();

        let result = run_rm(&config_path, &lockfile_path, "nonexistent");
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("not found"));
    }

    #[test]
    fn test_run_rm_without_lockfile() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join(".skem.yaml");
        let lockfile_path = temp_dir.path().join(".skem.lock");

        let config = Config {
            deps: vec![Dependency {
                name: "dep1".to_string(),
                repo: "https://github.com/example/dep1.git".to_string(),
                rev: None,
                paths: vec!["src/".to_string()],
                out: "./vendor/dep1".to_string(),
                hooks: vec![],
            }],
            post_hooks: vec![],
        };
        config::write_config(&config_path, &config).unwrap();

        // No lockfile exists, should still succeed
        run_rm(&config_path, &lockfile_path, "dep1").unwrap();

        let updated = config::read_config(&config_path).unwrap();
        let expected = Config {
            deps: vec![],
            post_hooks: vec![],
        };
        assert_eq!(updated, expected);
    }

    #[test]
    fn test_run_rm_config_not_found_fails() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join(".skem.yaml");
        let lockfile_path = temp_dir.path().join(".skem.lock");

        let result = run_rm(&config_path, &lockfile_path, "dep1");
        assert!(result.is_err());
    }
}
