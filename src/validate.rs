use crate::config::Config;
use anyhow::{bail, Result};
use std::collections::HashSet;

/// Validate a Config for semantic correctness
pub fn validate_config(config: &Config) -> Result<()> {
    // Validate each dependency field
    for (i, dep) in config.deps.iter().enumerate() {
        if dep.name.is_empty() {
            bail!("deps[{i}]: name must not be empty");
        }
        if dep.repo.is_empty() {
            bail!("deps[{i}] ({}): repo must not be empty", dep.name);
        }
        if dep.paths.is_empty() {
            bail!("deps[{i}] ({}): paths must not be empty", dep.name);
        }
        if dep.out.is_empty() {
            bail!("deps[{i}] ({}): out must not be empty", dep.name);
        }
    }

    // Check for duplicate dependency names
    let mut seen_names = HashSet::new();
    for dep in &config.deps {
        if !seen_names.insert(&dep.name) {
            bail!("duplicate dependency name: {}", dep.name);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, Dependency};

    #[test]
    fn test_validate_config_valid() {
        let config = Config {
            deps: vec![Dependency {
                name: "dep1".to_string(),
                repo: "https://github.com/example/repo.git".to_string(),
                rev: Some("main".to_string()),
                paths: vec!["src/".to_string()],
                out: "./vendor/dep1".to_string(),
                hooks: vec![],
            }],
            post_hooks: vec![],
        };

        let result = validate_config(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_config_empty_deps_is_valid() {
        let config = Config {
            deps: vec![],
            post_hooks: vec![],
        };

        let result = validate_config(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_config_duplicate_names() {
        let config = Config {
            deps: vec![
                Dependency {
                    name: "dep1".to_string(),
                    repo: "https://github.com/example/repo1.git".to_string(),
                    rev: None,
                    paths: vec!["src/".to_string()],
                    out: "./vendor/dep1".to_string(),
                    hooks: vec![],
                },
                Dependency {
                    name: "dep1".to_string(),
                    repo: "https://github.com/example/repo2.git".to_string(),
                    rev: None,
                    paths: vec!["lib/".to_string()],
                    out: "./vendor/dep2".to_string(),
                    hooks: vec![],
                },
            ],
            post_hooks: vec![],
        };

        let result = validate_config(&config);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("dep1"),
            "error should mention the duplicate name: {err_msg}"
        );
    }

    #[test]
    fn test_validate_config_empty_name() {
        let config = Config {
            deps: vec![Dependency {
                name: "".to_string(),
                repo: "https://github.com/example/repo.git".to_string(),
                rev: None,
                paths: vec!["src/".to_string()],
                out: "./vendor/dep1".to_string(),
                hooks: vec![],
            }],
            post_hooks: vec![],
        };

        let result = validate_config(&config);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("name"),
            "error should mention 'name': {err_msg}"
        );
    }

    #[test]
    fn test_validate_config_empty_repo() {
        let config = Config {
            deps: vec![Dependency {
                name: "dep1".to_string(),
                repo: "".to_string(),
                rev: None,
                paths: vec!["src/".to_string()],
                out: "./vendor/dep1".to_string(),
                hooks: vec![],
            }],
            post_hooks: vec![],
        };

        let result = validate_config(&config);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("repo"),
            "error should mention 'repo': {err_msg}"
        );
    }

    #[test]
    fn test_validate_config_empty_paths() {
        let config = Config {
            deps: vec![Dependency {
                name: "dep1".to_string(),
                repo: "https://github.com/example/repo.git".to_string(),
                rev: None,
                paths: vec![],
                out: "./vendor/dep1".to_string(),
                hooks: vec![],
            }],
            post_hooks: vec![],
        };

        let result = validate_config(&config);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("paths"),
            "error should mention 'paths': {err_msg}"
        );
    }

    #[test]
    fn test_validate_config_empty_out() {
        let config = Config {
            deps: vec![Dependency {
                name: "dep1".to_string(),
                repo: "https://github.com/example/repo.git".to_string(),
                rev: None,
                paths: vec!["src/".to_string()],
                out: "".to_string(),
                hooks: vec![],
            }],
            post_hooks: vec![],
        };

        let result = validate_config(&config);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("out"),
            "error should mention 'out': {err_msg}"
        );
    }

    #[test]
    fn test_validate_config_multiple_errors_reports_first() {
        let config = Config {
            deps: vec![Dependency {
                name: "".to_string(),
                repo: "".to_string(),
                rev: None,
                paths: vec![],
                out: "".to_string(),
                hooks: vec![],
            }],
            post_hooks: vec![],
        };

        let result = validate_config(&config);
        assert!(result.is_err());
    }
}
