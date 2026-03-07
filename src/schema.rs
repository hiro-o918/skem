use crate::config::Config;
use anyhow::Result;
use schemars::schema_for;

/// Generate and output JSON Schema for the Config structure
pub fn schema() -> Result<()> {
    let schema = schema_for!(Config);
    let json_output = serde_json::to_string_pretty(&schema)?;
    println!("{json_output}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn test_schema_generation() {
        // JSON Schema should be generated without errors
        let result = schema_for!(Config);
        let json_output = serde_json::to_string(&result).unwrap();
        assert!(!json_output.is_empty());
    }

    #[test]
    fn test_schema_has_required_properties() {
        let schema = schema_for!(Config);
        let schema_json = serde_json::to_value(&schema).unwrap();

        // Schema should have properties for Config
        assert!(schema_json.get("properties").is_some());
        assert!(schema_json.get("properties").unwrap().get("deps").is_some());
    }

    #[test]
    fn test_schema_output_is_valid_json() {
        let schema = schema_for!(Config);
        let json_output = serde_json::to_string(&schema).unwrap();

        // Should be parseable JSON
        let _parsed: Value = serde_json::from_str(&json_output).unwrap();
    }

    #[test]
    fn test_schema_pretty_print() {
        let schema = schema_for!(Config);
        let pretty_output = serde_json::to_string_pretty(&schema).unwrap();

        // Pretty printed output should contain newlines and indentation
        assert!(pretty_output.contains('\n'));
        assert!(pretty_output.contains("  "));
    }

    #[test]
    fn test_schema_includes_dependency_properties() {
        let schema = schema_for!(Config);
        let schema_json = serde_json::to_value(&schema).unwrap();

        // Dependency definition should exist in definitions
        let dependency_def = schema_json
            .get("definitions")
            .and_then(|d| d.get("Dependency"))
            .and_then(|d| d.get("properties"));

        // Dependency should have all required properties
        assert!(dependency_def.is_some());
        let props = dependency_def.unwrap();
        assert!(props.get("name").is_some());
        assert!(props.get("repo").is_some());
        assert!(props.get("rev").is_some());
        assert!(props.get("paths").is_some());
        assert!(props.get("out").is_some());
        assert!(props.get("hooks").is_some());
    }
}
