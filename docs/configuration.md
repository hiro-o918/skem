# Configuration

## Example

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

## Fields

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

## Hooks

### Per-dependency hooks

Per-dependency `hooks` run after each dependency is successfully synced. The following environment variables are available:

| Variable | Description |
|----------|-------------|
| `SKEM_SYNCED_FILES` | Space-separated list of file paths that were synced to the output directory |

Example:

```yaml
deps:
  - name: example-api
    repo: "https://github.com/example/api.git"
    paths:
      - "proto/"
    out: "./vendor/api"
    hooks:
      - "protoc --go_out=. $SKEM_SYNCED_FILES"
```

### Post hooks

Global `post_hooks` run once after all dependencies have been synced. No additional environment variables are provided.

```yaml
post_hooks:
  - "echo 'All dependencies synced'"
```
