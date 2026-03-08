use crate::config::{self, Dependency, Lockfile};
use crate::git::GitCommand;
use crate::lockfile;
use anyhow::Result;
use std::path::Path;

/// Result of checking a single dependency
#[derive(Debug, PartialEq)]
pub enum CheckResult {
    /// Dependency is up to date
    UpToDate,
    /// Dependency has an available update
    UpdateAvailable {
        current_sha: String,
        latest_sha: String,
    },
    /// Dependency is not yet synced
    NotSynced,
}

/// Check a single dependency for updates
///
/// # Arguments
/// * `dep` - Dependency to check
/// * `lockfile` - Current lockfile
///
/// # Returns
/// CheckResult indicating the status
pub fn check_dependency(dep: &Dependency, lockfile: &Lockfile) -> Result<CheckResult> {
    let rev = dep.rev.as_deref().unwrap_or("HEAD");
    let latest_sha = GitCommand::ls_remote(&dep.repo, rev)?;

    match lockfile.locks.iter().find(|entry| entry.name == dep.name) {
        Some(entry) => {
            if entry.sha == latest_sha {
                Ok(CheckResult::UpToDate)
            } else {
                Ok(CheckResult::UpdateAvailable {
                    current_sha: entry.sha.clone(),
                    latest_sha,
                })
            }
        }
        None => Ok(CheckResult::NotSynced),
    }
}

/// Check all dependencies for updates
///
/// # Arguments
/// * `config_path` - Path to config file
/// * `lockfile_path` - Path to lockfile
///
/// # Returns
/// true if all dependencies are up to date, false otherwise
pub fn run_check(config_path: &Path, lockfile_path: &Path) -> Result<bool> {
    let config = config::read_config(config_path)?;

    if config.deps.is_empty() {
        println!("No dependencies to check.");
        return Ok(true);
    }

    let current_lockfile = lockfile::read_lockfile(lockfile_path)?;
    let mut all_up_to_date = true;

    for dep in &config.deps {
        let result = check_dependency(dep, &current_lockfile)?;
        match result {
            CheckResult::UpToDate => {
                println!("  {} is up to date.", dep.name);
            }
            CheckResult::UpdateAvailable {
                current_sha,
                latest_sha,
            } => {
                println!(
                    "  {} has update available: {} -> {}",
                    dep.name,
                    &current_sha[..7.min(current_sha.len())],
                    &latest_sha[..7.min(latest_sha.len())]
                );
                all_up_to_date = false;
            }
            CheckResult::NotSynced => {
                println!("  {} is not synced yet.", dep.name);
                all_up_to_date = false;
            }
        }
    }

    if all_up_to_date {
        println!("All dependencies are up to date.");
    } else {
        println!("Some dependencies have updates available. Run 'skem sync' to update.");
    }

    Ok(all_up_to_date)
}

/// Run check with default paths
pub fn run_check_default() -> Result<bool> {
    run_check(
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
    fn test_check_dependency_up_to_date() {
        let dep = Dependency {
            name: "test".to_string(),
            repo: "https://github.com/hiro-o918/skem.git".to_string(),
            rev: Some("refs/heads/main".to_string()),
            paths: vec!["src/".to_string()],
            out: "./vendor/test".to_string(),
            hooks: vec![],
        };

        // Get current SHA
        let current_sha =
            GitCommand::ls_remote(&dep.repo, "refs/heads/main").expect("Should get SHA");

        let lockfile = Lockfile {
            locks: vec![LockEntry {
                name: "test".to_string(),
                sha: current_sha,
            }],
        };

        let result = check_dependency(&dep, &lockfile).unwrap();
        assert_eq!(result, CheckResult::UpToDate);
    }

    #[test]
    fn test_check_dependency_update_available() {
        let dep = Dependency {
            name: "test".to_string(),
            repo: "https://github.com/hiro-o918/skem.git".to_string(),
            rev: Some("refs/heads/main".to_string()),
            paths: vec!["src/".to_string()],
            out: "./vendor/test".to_string(),
            hooks: vec![],
        };

        let lockfile = Lockfile {
            locks: vec![LockEntry {
                name: "test".to_string(),
                sha: "0000000000000000000000000000000000000000".to_string(),
            }],
        };

        let result = check_dependency(&dep, &lockfile).unwrap();
        match &result {
            CheckResult::UpdateAvailable {
                current_sha,
                latest_sha,
            } => {
                assert_eq!(
                    current_sha, "0000000000000000000000000000000000000000",
                    "current_sha should be the dummy SHA from lockfile"
                );
                assert!(
                    !latest_sha.is_empty(),
                    "latest_sha should be a non-empty SHA from remote"
                );
                assert_ne!(
                    latest_sha, current_sha,
                    "latest_sha should differ from current_sha"
                );
            }
            other => panic!("Expected UpdateAvailable, got {other:?}"),
        }
    }

    #[test]
    fn test_check_dependency_not_synced() {
        let dep = Dependency {
            name: "test".to_string(),
            repo: "https://github.com/hiro-o918/skem.git".to_string(),
            rev: Some("refs/heads/main".to_string()),
            paths: vec!["src/".to_string()],
            out: "./vendor/test".to_string(),
            hooks: vec![],
        };

        let lockfile = Lockfile { locks: vec![] };

        let result = check_dependency(&dep, &lockfile).unwrap();
        assert_eq!(result, CheckResult::NotSynced);
    }

    #[test]
    fn test_run_check_empty_deps() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join(".skem.yaml");
        let lockfile_path = temp_dir.path().join(".skem.lock");

        let config = Config { deps: vec![] };
        config::write_config(&config_path, &config).unwrap();

        let result = run_check(&config_path, &lockfile_path).unwrap();
        assert!(result);
    }

    #[test]
    fn test_run_check_config_not_found_fails() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join(".skem.yaml");
        let lockfile_path = temp_dir.path().join(".skem.lock");

        let result = run_check(&config_path, &lockfile_path);
        assert!(result.is_err());
    }
}
