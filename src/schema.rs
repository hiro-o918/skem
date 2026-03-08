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
}
