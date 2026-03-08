use crate::config::Config;
use anyhow::Result;
use schemars::schema_for;

/// Generate JSON Schema string for the Config structure
pub fn generate_schema_string() -> Result<String> {
    let schema = schema_for!(Config);
    Ok(serde_json::to_string_pretty(&schema)?)
}

/// Generate and output JSON Schema for the Config structure
pub fn schema() -> Result<()> {
    let schema_string = generate_schema_string()?;
    println!("{schema_string}");
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_generate_schema_string_returns_valid_json() {
        let schema_string = super::generate_schema_string().unwrap();
        let schema: serde_json::Value = serde_json::from_str(&schema_string).unwrap();

        // Top-level structure
        assert_eq!(schema["title"], "Config");
        assert_eq!(schema["type"], "object");

        // Dependency definition
        let dep = &schema["definitions"]["Dependency"];
        assert_eq!(dep["type"], "object");

        // rev should not be in required list
        let required = dep["required"].as_array().unwrap();
        let required_names: Vec<&str> = required.iter().map(|v| v.as_str().unwrap()).collect();
        assert!(required_names.contains(&"name"));
        assert!(required_names.contains(&"repo"));
        assert!(required_names.contains(&"paths"));
        assert!(required_names.contains(&"out"));
        assert!(!required_names.contains(&"rev"));

        // rev property should exist
        assert!(dep["properties"]["rev"].is_object());
    }

    #[test]
    fn test_config_deserialize_with_post_hooks() {
        // Arrange: Config with post_hooks
        let yaml = r#"
deps:
  - name: example
    repo: "https://github.com/example/repo.git"
    rev: "main"
    paths:
      - "proto/"
    out: "./vendor"
post_hooks:
  - "echo 'All dependencies synced'"
  - "cargo fmt"
"#;

        // Act: Deserialize YAML to Config
        let config: crate::config::Config = serde_yaml::from_str(yaml).unwrap();

        // Assert: Compare complete Config object
        let expected = crate::config::Config {
            deps: vec![crate::config::Dependency {
                name: "example".to_string(),
                repo: "https://github.com/example/repo.git".to_string(),
                rev: Some("main".to_string()),
                paths: vec!["proto/".to_string()],
                out: "./vendor".to_string(),
                hooks: vec![],
            }],
            post_hooks: vec![
                "echo 'All dependencies synced'".to_string(),
                "cargo fmt".to_string(),
            ],
        };
        assert_eq!(config, expected);
    }

    #[test]
    fn test_config_deserialize_without_post_hooks() {
        // Arrange: Config without post_hooks (should default to empty array)
        let yaml = r#"
deps:
  - name: example
    repo: "https://github.com/example/repo.git"
    rev: "main"
    paths:
      - "proto/"
    out: "./vendor"
"#;

        // Act: Deserialize YAML to Config
        let config: crate::config::Config = serde_yaml::from_str(yaml).unwrap();

        // Assert: post_hooks should be empty
        let expected = crate::config::Config {
            deps: vec![crate::config::Dependency {
                name: "example".to_string(),
                repo: "https://github.com/example/repo.git".to_string(),
                rev: Some("main".to_string()),
                paths: vec!["proto/".to_string()],
                out: "./vendor".to_string(),
                hooks: vec![],
            }],
            post_hooks: vec![],
        };
        assert_eq!(config, expected);
    }
}
