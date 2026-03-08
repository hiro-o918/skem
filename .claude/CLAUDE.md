# Project-specific Instructions for skem

## Language Guidelines

- **Code Comments**: Write all code comments in English since this is an OSS project
- **Documentation**: Write all documentation (README, doc comments, etc.) in English
- **Commit Messages**: Use English for commit messages following Conventional Commits format
- **Pull Requests**: Write PR titles in English following Conventional Commits format, and descriptions in English
- **Communication with User**: Use Japanese for communication with the user (as per global settings)

## Development Guidelines

- Follow TDD (Test-Driven Development) approach
- Write tests before implementation
- **Before committing, ensure all of the following pass:**
  - `cargo test` - Run all tests
  - `cargo clippy --all-targets --all-features` - Lint check
  - `cargo fmt --all --check` - Format check
- Use `cargo fmt --all` to auto-format code

## Testing (TDD Approach)

### Test-Driven Development Flow

Follow the t-wada style TDD cycle:
1. **Red**: Write failing test first
2. **Green**: Implement minimal code to make test pass
3. **Refactor**: Improve code while tests remain passing

### Test Writing Guidelines

- **Arrange-Act-Assert Pattern**: Structure each test with clear setup, action, and assertion
- **Value Object Comparison**: For data structures/models, always compare complete objects using `assert_eq!(actual, expected)`, not individual fields
  - **Good**: `assert_eq!(config, expected_config);`
  - **Bad**: `assert!(config.name.is_some());`
- **Test Data in Functions**: Include test data directly in test functions (not in fixtures unless it's assertion-unrelated)
- **Minimal Mocking**: Avoid mocks; only use them for external dependencies (API calls, DB calls)
- **Meaningful Test Names**: Use `test_<functionality>_<condition>_<expected_outcome>` format
- **Single Responsibility**: Each test should verify one behavior

### Testing Data Models

- Test serialization and deserialization separately
- For YAML/JSON round-trips, verify complete object equality
- Include tests with and without optional fields
- Test with multiple values to ensure correctness

### Example Test Pattern

```rust
#[test]
fn test_config_deserialize_with_all_fields() {
    // Arrange: Prepare test data
    let yaml = r#"
deps:
  - name: example
    repo: "https://github.com/example/repo.git"
    rev: "main"
    paths:
      - "proto/"
    out: "./vendor"
    hooks:
      - "echo 'done'"
"#;

    // Act: Execute the function being tested
    let config: Config = serde_yaml::from_str(yaml).unwrap();

    // Assert: Compare complete object, not individual fields
    let expected = Config {
        deps: vec![Dependency {
            name: "example".to_string(),
            repo: "https://github.com/example/repo.git".to_string(),
            rev: "main".to_string(),
            paths: vec!["proto/".to_string()],
            out: "./vendor".to_string(),
            hooks: vec!["echo 'done'".to_string()],
        }],
    };
    assert_eq!(config, expected);
}
```

### Test Coverage Expectations

- Write unit tests for all modules
- Test both serialization and deserialization for data models
- Ensure comprehensive test coverage for core functionality
- Each feature should have corresponding tests before implementation
