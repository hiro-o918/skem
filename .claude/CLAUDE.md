# Project-specific Instructions for skem

## Language Guidelines

- **Code Comments**: Write all code comments in English since this is an OSS project
- **Documentation**: Write all documentation (README, doc comments, etc.) in English
- **Commit Messages**: Use English for commit messages following Conventional Commits format
- **Communication with User**: Use Japanese for communication with the user (as per global settings)

## Development Guidelines

- Follow TDD (Test-Driven Development) approach
- Write tests before implementation
- **Before committing, ensure all of the following pass:**
  - `cargo test` - Run all tests
  - `cargo clippy --all-targets --all-features` - Lint check
  - `cargo fmt --all --check` - Format check
- Use `cargo fmt --all` to auto-format code

## Testing

- Write unit tests for all modules
- Test both serialization and deserialization for data models
- Ensure comprehensive test coverage
