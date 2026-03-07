use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Structure of .skem.yaml configuration file
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Config {
    /// List of dependencies
    pub deps: Vec<Dependency>,
}

/// Individual dependency definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Dependency {
    /// Name of the dependency
    pub name: String,
    /// Git repository URL
    pub repo: String,
    /// Branch, tag, or commit hash
    pub rev: String,
    /// List of paths to download
    pub paths: Vec<String>,
    /// Output directory
    pub out: String,
    /// Commands to execute when changes are detected
    #[serde(default)]
    pub hooks: Vec<String>,
}

/// Structure of .skem.lock file
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Lockfile {
    /// List of locked dependencies
    pub locks: Vec<LockEntry>,
}

/// Individual entry in the lock file
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct LockEntry {
    /// Name of the dependency
    pub name: String,
    /// Resolved commit SHA
    pub sha: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_deserialize() {
        let yaml = r#"
deps:
  - name: example-api
    repo: "https://github.com/example/api.git"
    rev: "main"
    paths:
      - "proto/v1/"
    out: "./vendor/api"
    hooks:
      - "echo 'Files updated'"
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        let expected = Config {
            deps: vec![Dependency {
                name: "example-api".to_string(),
                repo: "https://github.com/example/api.git".to_string(),
                rev: "main".to_string(),
                paths: vec!["proto/v1/".to_string()],
                out: "./vendor/api".to_string(),
                hooks: vec!["echo 'Files updated'".to_string()],
            }],
        };
        assert_eq!(config, expected);
    }

    #[test]
    fn test_config_deserialize_without_hooks() {
        let yaml = r#"
deps:
  - name: test-dep
    repo: "https://github.com/test/repo.git"
    rev: "v1.0.0"
    paths:
      - "src/"
    out: "./vendor/test"
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        let expected = Config {
            deps: vec![Dependency {
                name: "test-dep".to_string(),
                repo: "https://github.com/test/repo.git".to_string(),
                rev: "v1.0.0".to_string(),
                paths: vec!["src/".to_string()],
                out: "./vendor/test".to_string(),
                hooks: vec![],
            }],
        };
        assert_eq!(config, expected);
    }

    #[test]
    fn test_config_serialize() {
        let config = Config {
            deps: vec![Dependency {
                name: "test-dep".to_string(),
                repo: "https://github.com/test/repo.git".to_string(),
                rev: "main".to_string(),
                paths: vec!["src/".to_string()],
                out: "./vendor".to_string(),
                hooks: vec!["echo 'done'".to_string()],
            }],
        };
        let yaml = serde_yaml::to_string(&config).unwrap();
        let deserialized: Config = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(deserialized, config);
    }

    #[test]
    fn test_lockfile_deserialize() {
        let yaml = r#"
locks:
  - name: example-api
    sha: "abc123def456"
  - name: another-dep
    sha: "789ghi012jkl"
"#;
        let lockfile: Lockfile = serde_yaml::from_str(yaml).unwrap();
        let expected = Lockfile {
            locks: vec![
                LockEntry {
                    name: "example-api".to_string(),
                    sha: "abc123def456".to_string(),
                },
                LockEntry {
                    name: "another-dep".to_string(),
                    sha: "789ghi012jkl".to_string(),
                },
            ],
        };
        assert_eq!(lockfile, expected);
    }

    #[test]
    fn test_lockfile_serialize() {
        let lockfile = Lockfile {
            locks: vec![
                LockEntry {
                    name: "dep1".to_string(),
                    sha: "sha1".to_string(),
                },
                LockEntry {
                    name: "dep2".to_string(),
                    sha: "sha2".to_string(),
                },
            ],
        };
        let yaml = serde_yaml::to_string(&lockfile).unwrap();
        let deserialized: Lockfile = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(deserialized, lockfile);
    }

    #[test]
    fn test_multiple_dependencies() {
        let yaml = r#"
deps:
  - name: dep1
    repo: "https://github.com/user1/repo1.git"
    rev: "v1.0.0"
    paths:
      - "path1/"
    out: "./vendor/dep1"
  - name: dep2
    repo: "https://github.com/user2/repo2.git"
    rev: "main"
    paths:
      - "path2/"
      - "path3/"
    out: "./vendor/dep2"
    hooks:
      - "echo 'updated'"
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        let expected = Config {
            deps: vec![
                Dependency {
                    name: "dep1".to_string(),
                    repo: "https://github.com/user1/repo1.git".to_string(),
                    rev: "v1.0.0".to_string(),
                    paths: vec!["path1/".to_string()],
                    out: "./vendor/dep1".to_string(),
                    hooks: vec![],
                },
                Dependency {
                    name: "dep2".to_string(),
                    repo: "https://github.com/user2/repo2.git".to_string(),
                    rev: "main".to_string(),
                    paths: vec!["path2/".to_string(), "path3/".to_string()],
                    out: "./vendor/dep2".to_string(),
                    hooks: vec!["echo 'updated'".to_string()],
                },
            ],
        };
        assert_eq!(config, expected);
    }
}
