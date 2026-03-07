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
    fn test_generate_schema_string_returns_correct_schema() {
        let expected = r##"{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Config",
  "description": "Structure of .skem.yaml configuration file",
  "type": "object",
  "required": [
    "deps"
  ],
  "properties": {
    "deps": {
      "description": "List of dependencies",
      "type": "array",
      "items": {
        "$ref": "#/definitions/Dependency"
      }
    }
  },
  "definitions": {
    "Dependency": {
      "description": "Individual dependency definition",
      "type": "object",
      "required": [
        "name",
        "out",
        "paths",
        "repo",
        "rev"
      ],
      "properties": {
        "hooks": {
          "description": "Commands to execute when changes are detected",
          "default": [],
          "type": "array",
          "items": {
            "type": "string"
          }
        },
        "name": {
          "description": "Name of the dependency",
          "type": "string"
        },
        "out": {
          "description": "Output directory",
          "type": "string"
        },
        "paths": {
          "description": "List of paths to download",
          "type": "array",
          "items": {
            "type": "string"
          }
        },
        "repo": {
          "description": "Git repository URL",
          "type": "string"
        },
        "rev": {
          "description": "Branch, tag, or commit hash",
          "type": "string"
        }
      }
    }
  }
}"##;

        let actual = super::generate_schema_string().unwrap();
        assert_eq!(actual, expected);
    }
}
