# skem

A lightweight CLI tool to download specific files from remote Git repositories and run hooks on changes.

## Features

- 📦 Download specific files/directories from Git repositories using sparse checkout
- 🔄 Track changes using lockfile mechanism
- 🪝 Run custom hooks when files are updated
- ⚡ Parallel synchronization of multiple dependencies
- 🎯 Minimal overhead with efficient Git operations

## Installation

```bash
cargo install skem
```

## Usage

### Initialize configuration

```bash
skem init
```

This creates a `.skem.yaml` file in the current directory with an example configuration.

### Synchronize dependencies

```bash
skem sync
```

Or simply:

```bash
skem
```

This downloads files from configured repositories and runs hooks if there are changes.

### Generate JSON Schema

```bash
skem schema
```

Outputs JSON Schema for `.skem.yaml` configuration file, useful for editor completion.

## Configuration

Example `.skem.yaml`:

```yaml
deps:
  - name: example-api
    repo: "https://github.com/example/api.git"
    rev: "main"
    paths:
      - "proto/v1/"
    out: "./vendor/api"
    hooks:
      - "echo 'Files updated'"
```

### Configuration fields

- `name`: Dependency name (for identification)
- `repo`: Git repository URL
- `rev`: Branch, tag, or commit SHA
- `paths`: List of paths to download (supports sparse checkout)
- `out`: Output directory
- `hooks`: Commands to run after successful synchronization (optional)

## How it works

1. Reads `.skem.yaml` configuration
2. For each dependency:
   - Checks remote commit SHA using `git ls-remote`
   - Compares with `.skem.lock` to detect changes
   - If changed, downloads files using sparse checkout
   - Strips path prefixes and copies to output directory
   - Runs hooks if specified
   - Updates lockfile on success
3. All dependencies are processed in parallel

## License

MIT OR Apache-2.0

## Author

Hironori Yamamoto <hiro.o918@gmail.com>
