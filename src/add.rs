use crate::config::{self, Config, Dependency};
use anyhow::{anyhow, Result};
use std::path::Path;

/// Extract repository name from URL
///
/// # Examples
/// - `https://github.com/example/api.git` → `api`
/// - `https://github.com/example/api` → `api`
/// - `git@github.com:example/api.git` → `api`
pub fn extract_repo_name(repo: &str) -> Result<String> {
    let name = repo
        .trim_end_matches('/')
        .trim_end_matches(".git")
        .rsplit('/')
        .next()
        // Handle ssh-style URLs (git@github.com:example/api.git)
        .or_else(|| repo.trim_end_matches(".git").rsplit(':').next())
        .ok_or_else(|| anyhow!("Failed to extract repository name from '{repo}'"))?;

    if name.is_empty() {
        anyhow::bail!("Failed to extract repository name from '{repo}'");
    }

    Ok(name.to_string())
}

/// Add a new dependency to .skem.yaml
///
/// # Arguments
/// * `config_path` - Path to the config file
/// * `repo` - Git repository URL
/// * `paths` - List of paths to download
/// * `out` - Output directory
/// * `name` - Optional dependency name (extracted from repo URL if omitted)
/// * `rev` - Optional revision (branch, tag, or commit)
pub fn run_add(
    config_path: &Path,
    repo: &str,
    paths: Vec<String>,
    out: &str,
    name: Option<&str>,
    rev: Option<&str>,
) -> Result<()> {
    let dep_name = match name {
        Some(n) => n.to_string(),
        None => extract_repo_name(repo)?,
    };

    let mut config = if config_path.exists() {
        config::read_config(config_path)?
    } else {
        Config { deps: vec![] }
    };

    // Check for duplicate name
    if config.deps.iter().any(|d| d.name == dep_name) {
        anyhow::bail!("Dependency '{dep_name}' already exists in the configuration.");
    }

    let dependency = Dependency {
        name: dep_name.clone(),
        repo: repo.to_string(),
        rev: rev.map(|r| r.to_string()),
        paths,
        out: out.to_string(),
        hooks: vec![],
    };

    config.deps.push(dependency);
    config::write_config(config_path, &config)?;

    println!("Added dependency '{dep_name}'.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_extract_repo_name_https_with_git_suffix() {
        let result = extract_repo_name("https://github.com/example/api.git");
        assert_eq!(result.unwrap(), "api");
    }

    #[test]
    fn test_extract_repo_name_https_without_git_suffix() {
        let result = extract_repo_name("https://github.com/example/api");
        assert_eq!(result.unwrap(), "api");
    }

    #[test]
    fn test_extract_repo_name_with_trailing_slash() {
        let result = extract_repo_name("https://github.com/example/api/");
        assert_eq!(result.unwrap(), "api");
    }

    #[test]
    fn test_run_add_creates_new_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join(".skem.yaml");

        run_add(
            &config_path,
            "https://github.com/example/api.git",
            vec!["proto/".to_string()],
            "./vendor/api",
            None,
            Some("main"),
        )
        .unwrap();

        let config = config::read_config(&config_path).unwrap();
        let expected = Config {
            deps: vec![Dependency {
                name: "api".to_string(),
                repo: "https://github.com/example/api.git".to_string(),
                rev: Some("main".to_string()),
                paths: vec!["proto/".to_string()],
                out: "./vendor/api".to_string(),
                hooks: vec![],
            }],
        };
        assert_eq!(config, expected);
    }

    #[test]
    fn test_run_add_with_explicit_name() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join(".skem.yaml");

        run_add(
            &config_path,
            "https://github.com/example/api.git",
            vec!["proto/".to_string()],
            "./vendor/api",
            Some("my-api"),
            None,
        )
        .unwrap();

        let config = config::read_config(&config_path).unwrap();
        let expected = Config {
            deps: vec![Dependency {
                name: "my-api".to_string(),
                repo: "https://github.com/example/api.git".to_string(),
                rev: None,
                paths: vec!["proto/".to_string()],
                out: "./vendor/api".to_string(),
                hooks: vec![],
            }],
        };
        assert_eq!(config, expected);
    }

    #[test]
    fn test_run_add_appends_to_existing_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join(".skem.yaml");

        let initial_config = Config {
            deps: vec![Dependency {
                name: "existing".to_string(),
                repo: "https://github.com/example/existing.git".to_string(),
                rev: Some("v1.0".to_string()),
                paths: vec!["src/".to_string()],
                out: "./vendor/existing".to_string(),
                hooks: vec![],
            }],
        };
        config::write_config(&config_path, &initial_config).unwrap();

        run_add(
            &config_path,
            "https://github.com/example/new.git",
            vec!["proto/".to_string()],
            "./vendor/new",
            None,
            None,
        )
        .unwrap();

        let config = config::read_config(&config_path).unwrap();
        let expected = Config {
            deps: vec![
                Dependency {
                    name: "existing".to_string(),
                    repo: "https://github.com/example/existing.git".to_string(),
                    rev: Some("v1.0".to_string()),
                    paths: vec!["src/".to_string()],
                    out: "./vendor/existing".to_string(),
                    hooks: vec![],
                },
                Dependency {
                    name: "new".to_string(),
                    repo: "https://github.com/example/new.git".to_string(),
                    rev: None,
                    paths: vec!["proto/".to_string()],
                    out: "./vendor/new".to_string(),
                    hooks: vec![],
                },
            ],
        };
        assert_eq!(config, expected);
    }

    #[test]
    fn test_run_add_duplicate_name_fails() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join(".skem.yaml");

        let initial_config = Config {
            deps: vec![Dependency {
                name: "api".to_string(),
                repo: "https://github.com/example/api.git".to_string(),
                rev: None,
                paths: vec!["proto/".to_string()],
                out: "./vendor/api".to_string(),
                hooks: vec![],
            }],
        };
        config::write_config(&config_path, &initial_config).unwrap();

        let result = run_add(
            &config_path,
            "https://github.com/example/api.git",
            vec!["src/".to_string()],
            "./vendor/api2",
            Some("api"),
            None,
        );

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("already exists"));
    }

    #[test]
    fn test_run_add_without_rev() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join(".skem.yaml");

        run_add(
            &config_path,
            "https://github.com/example/api.git",
            vec!["proto/".to_string()],
            "./vendor/api",
            None,
            None,
        )
        .unwrap();

        let config = config::read_config(&config_path).unwrap();
        let expected = Config {
            deps: vec![Dependency {
                name: "api".to_string(),
                repo: "https://github.com/example/api.git".to_string(),
                rev: None,
                paths: vec!["proto/".to_string()],
                out: "./vendor/api".to_string(),
                hooks: vec![],
            }],
        };
        assert_eq!(config, expected);

        // Verify rev is not serialized in YAML
        let yaml = fs::read_to_string(&config_path).unwrap();
        assert!(!yaml.contains("rev:"));
    }

    #[test]
    fn test_run_add_multiple_paths() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join(".skem.yaml");

        run_add(
            &config_path,
            "https://github.com/example/api.git",
            vec!["proto/".to_string(), "openapi/".to_string()],
            "./vendor/api",
            None,
            Some("v2.0"),
        )
        .unwrap();

        let config = config::read_config(&config_path).unwrap();
        let expected = Config {
            deps: vec![Dependency {
                name: "api".to_string(),
                repo: "https://github.com/example/api.git".to_string(),
                rev: Some("v2.0".to_string()),
                paths: vec!["proto/".to_string(), "openapi/".to_string()],
                out: "./vendor/api".to_string(),
                hooks: vec![],
            }],
        };
        assert_eq!(config, expected);
    }
}
