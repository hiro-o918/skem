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
    use super::*;

    #[test]
    fn test_generate_schema_string_produces_valid_json() {
        // Arrange
        let expected_schema_1 = schema_for!(Config);

        // Act
        let schema_string = generate_schema_string().unwrap();

        // Assert: Output should be valid JSON that matches the schema
        let parsed_schema: serde_json::Value = serde_json::from_str(&schema_string).unwrap();
        let expected_value = serde_json::to_value(&expected_schema_1).unwrap();
        assert_eq!(parsed_schema, expected_value);
    }

    #[test]
    fn test_generate_schema_string_is_deterministic() {
        // Arrange
        let schema_string_1 = generate_schema_string().unwrap();

        // Act
        let schema_string_2 = generate_schema_string().unwrap();

        // Assert: Multiple calls should produce identical output strings
        assert_eq!(schema_string_1, schema_string_2);
    }
}
