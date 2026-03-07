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

    #[test]
    fn test_schema_output_is_valid_json_schema() {
        let schema = schema_for!(Config);
        let json_output = serde_json::to_string_pretty(&schema).unwrap();

        // Verify that output can be parsed back as JSON and contains required schema fields
        let parsed: serde_json::Value = serde_json::from_str(&json_output).unwrap();
        assert_eq!(parsed.get("title").and_then(|v| v.as_str()), Some("Config"));
        assert!(parsed.get("properties").is_some());
        assert!(parsed.get("definitions").is_some());
    }

    #[test]
    fn test_schema_includes_all_dependency_properties() {
        let schema = schema_for!(Config);
        let schema_json = serde_json::to_value(&schema).unwrap();

        let dependency_properties = schema_json
            .get("definitions")
            .and_then(|d| d.get("Dependency"))
            .and_then(|d| d.get("properties"))
            .unwrap();

        // Verify all required properties are present in Dependency
        assert!(dependency_properties.get("name").is_some());
        assert!(dependency_properties.get("repo").is_some());
        assert!(dependency_properties.get("rev").is_some());
        assert!(dependency_properties.get("paths").is_some());
        assert!(dependency_properties.get("out").is_some());
        assert!(dependency_properties.get("hooks").is_some());
    }

    #[test]
    fn test_schema_serialization_roundtrip() {
        let schema = schema_for!(Config);

        // Verify schema can be serialized and deserialized
        let json_str = serde_json::to_string(&schema).unwrap();
        let _reparsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

        let pretty_str = serde_json::to_string_pretty(&schema).unwrap();
        let _reparsed_pretty: serde_json::Value = serde_json::from_str(&pretty_str).unwrap();

        // Both serializations should succeed
        assert!(!json_str.is_empty());
        assert!(!pretty_str.is_empty());
    }
}
