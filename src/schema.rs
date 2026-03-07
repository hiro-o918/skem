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
    fn test_config_schema_generation_is_deterministic() {
        // Arrange
        let schema_1 = schema_for!(Config);

        // Act
        let schema_2 = schema_for!(Config);

        // Assert: Multiple calls should produce identical schemas
        assert_eq!(schema_1, schema_2);
    }

    #[test]
    fn test_config_schema_serializes_to_valid_json() {
        // Arrange
        let schema = schema_for!(Config);

        // Act: Serialize to JSON string
        let json_string = serde_json::to_string_pretty(&schema).unwrap();

        // Assert: Should deserialize back to the same schema object
        let parsed_schema: serde_json::Value = serde_json::from_str(&json_string).unwrap();
        let original_value = serde_json::to_value(&schema).unwrap();
        assert_eq!(parsed_schema, original_value);
    }
}
