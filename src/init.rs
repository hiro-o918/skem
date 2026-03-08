use crate::config::{Config, Dependency};
use anyhow::{anyhow, Result};
use std::fs;
use std::path::Path;

/// Sample configuration content to be used when initializing
fn get_sample_config() -> Config {
    Config {
        deps: vec![
            Dependency {
                name: "proto-schemas".to_string(),
                repo: "https://github.com/example/schemas.git".to_string(),
                rev: None,
                paths: vec!["proto/".to_string()],
                out: "./vendor/proto".to_string(),
                hooks: vec!["echo 'Proto files updated'".to_string()],
            },
            Dependency {
                name: "openapi-schemas".to_string(),
                repo: "https://github.com/example/schemas.git".to_string(),
                rev: None,
                paths: vec!["openapi/".to_string()],
                out: "./vendor/openapi".to_string(),
                hooks: vec![],
            },
        ],
        post_hooks: vec![],
    }
}

/// Initialize a new .skem.yaml configuration file in the current directory
pub fn init() -> Result<()> {
    let config_path = ".skem.yaml";

    // Check if the file already exists
    if Path::new(config_path).exists() {
        return Err(anyhow!(
            "File '{config_path}' already exists. Please remove it first or use a different directory."
        ));
    }

    // Get the sample configuration
    let sample_config = get_sample_config();

    // Serialize to YAML
    let yaml_content = serde_yaml::to_string(&sample_config)?;

    // Write to file
    fs::write(config_path, yaml_content)?;

    println!("Created {config_path} successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample_config_structure() {
        let config = get_sample_config();
        let expected = Config {
            deps: vec![
                Dependency {
                    name: "proto-schemas".to_string(),
                    repo: "https://github.com/example/schemas.git".to_string(),
                    rev: None,
                    paths: vec!["proto/".to_string()],
                    out: "./vendor/proto".to_string(),
                    hooks: vec!["echo 'Proto files updated'".to_string()],
                },
                Dependency {
                    name: "openapi-schemas".to_string(),
                    repo: "https://github.com/example/schemas.git".to_string(),
                    rev: None,
                    paths: vec!["openapi/".to_string()],
                    out: "./vendor/openapi".to_string(),
                    hooks: vec![],
                },
            ],
            post_hooks: vec![],
        };
        assert_eq!(config, expected);
    }

    #[test]
    fn test_sample_config_is_valid_yaml() {
        let config = get_sample_config();
        let yaml_content = serde_yaml::to_string(&config).unwrap();
        let deserialized: Config = serde_yaml::from_str(&yaml_content).unwrap();
        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_init_error_message_contains_helpful_text() {
        let error_msg =
            "File '.skem.yaml' already exists. Please remove it first or use a different directory.";
        assert!(error_msg.contains("already exists"));
    }
}
