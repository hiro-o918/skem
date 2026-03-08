# skem

A lightweight CLI tool to download specific files from remote Git repositories and run hooks on changes.

## Features

- Download specific files/directories from Git repositories using sparse checkout
- Track changes using a lockfile mechanism
- Run custom hooks when files are updated (per-dependency and global post-sync)
- Parallel synchronization of multiple dependencies
- Interactive mode for adding dependencies
- Minimal overhead with efficient Git operations

## Installation

```bash
cargo install skem
```

## Usage

### Initialize configuration

```bash
skem init
```

Creates a `.skem.yaml` file in the current directory with an example configuration.

### Synchronize dependencies

```bash
skem sync
```

Downloads files from configured repositories and runs hooks if there are changes.

### Add a dependency

```bash
skem add --repo https://github.com/example/api.git --paths proto/ --out ./vendor/api
```

You can also specify an optional name and revision:

```bash
skem add --repo https://github.com/example/api.git --paths proto/ openapi/ --out ./vendor/api --name my-api --rev v2.0
```

#### Interactive mode

Omit `--paths` and `--out` to enter interactive mode, which lets you browse the repository and select files:

```bash
skem add --repo https://github.com/example/api.git
```

### Remove a dependency

```bash
skem rm <name>
```

Removes a dependency from both `.skem.yaml` and `.skem.lock`.

### List dependencies

```bash
skem ls
```

Lists all configured dependencies with their sync status.

### Check for updates

```bash
skem check
```

Checks if any dependencies have updates available compared to the lockfile. Exits with code 1 if updates are found.

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
post_hooks:
  - "echo 'All dependencies synced'"
```

### Configuration fields

| Field | Required | Description |
|-------|----------|-------------|
| `deps` | Yes | List of dependencies |
| `deps[].name` | Yes | Dependency name (for identification) |
| `deps[].repo` | Yes | Git repository URL |
| `deps[].rev` | No | Branch, tag, or commit SHA (defaults to HEAD) |
| `deps[].paths` | Yes | List of paths to download (supports sparse checkout) |
| `deps[].out` | Yes | Output directory |
| `deps[].hooks` | No | Commands to run after successful synchronization |
| `post_hooks` | No | Commands to run after all dependencies are synced |

## How it works

1. Reads `.skem.yaml` configuration
2. For each dependency (processed in parallel):
   - Checks remote commit SHA using `git ls-remote`
   - Compares with `.skem.lock` to detect changes
   - If changed, downloads files using sparse checkout
   - Strips path prefixes and copies to output directory
   - Runs per-dependency hooks if specified
   - Updates lockfile on success
3. Runs global `post_hooks` after all dependencies are processed

## License

MIT

## Author

Hironori Yamamoto
