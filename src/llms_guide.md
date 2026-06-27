# skem — LLM Usage Guide

This document is the authoritative quick reference for an LLM agent
that needs to drive the `skem` CLI. It is self-contained: an agent
should be able to perform every common task from this text alone.

`skem` is a lightweight CLI that downloads specific files or directories
from remote Git repositories using sparse checkout, tracks their state
in a lockfile, and runs hooks when downloaded files change.

## Mental model

- `.skem.yaml` declares dependencies: which repo, which paths inside it,
  where to copy them, and which commands to run when they change.
- `.skem.lock` records the commit SHA last synced for each dependency.
  It is generated and updated by `skem` and should be committed to VCS.
- `skem sync` reconciles the working tree with `.skem.yaml`, updating
  files and `.skem.lock` and running hooks when something changed.
- All other subcommands are conveniences around editing `.skem.yaml`,
  inspecting state, or querying remotes.

## Subcommands

### `skem init`

Create a `.skem.yaml` template in the current directory. Fails if the
file already exists. Use this once at the start of a project.

### `skem schema`

Print the JSON Schema for `.skem.yaml` to stdout. Pipe this into an
editor schema store or use it to validate a configuration file.

### `skem sync`

Reconcile every dependency in `.skem.yaml`:

1. Resolve the remote commit SHA for each dependency's `rev`.
2. Compare against `.skem.lock` to decide whether anything changed.
3. For changed dependencies, sparse-checkout the configured `paths`
   and copy them into `out`.
4. Run that dependency's `hooks` (only when files actually changed).
5. After all dependencies are processed, run top-level `post_hooks`.

Flags:

- `--force` — re-sync every dependency even when the lockfile already
  matches the remote SHA. Use this when output files were modified
  locally and you want to restore them.
- `--hooks-only` — skip fetching, but still run `hooks` and
  `post_hooks`. Useful when a hook command itself was edited and you
  want to re-run it without touching the vendored files.

`--force` and `--hooks-only` are mutually exclusive at runtime.

### `skem add`

Append a new dependency to `.skem.yaml`. Two modes:

- **Interactive (recommended)**: omit `--paths` and `--out`. The CLI
  clones the repo, lets the user browse the tree, and prompts for the
  output directory.

  ```
  skem add --repo https://github.com/example/api.git
  ```

- **Full specification**: pass everything on the command line.

  ```
  skem add --repo https://github.com/example/api.git \
           --paths proto/ openapi/ \
           --out ./vendor/api \
           --name my-api \
           --rev v2.0
  ```

Flags:

- `--repo <URL>` (required) — Git repository URL.
- `--paths <PATH>...` — one or more paths to sparse-checkout.
- `--out <DIR>` — output directory.
- `--name <NAME>` — dependency name (defaults to the repo's basename).
- `--rev <REV>` — branch, tag, or commit SHA (defaults to the remote
  default branch).

`--paths` and `--out` must be specified together, or both omitted to
enter interactive mode.

`skem add` only edits `.skem.yaml`; run `skem sync` afterwards to
actually fetch the files.

### `skem rm <name>`

Remove the dependency named `<name>` from `.skem.yaml` and its entry
from `.skem.lock`. Does not delete the previously vendored files in
`out/`; remove them manually if desired.

### `skem ls`

List every dependency in `.skem.yaml` with its current sync status
relative to `.skem.lock`.

### `skem check`

Query each dependency's remote and report whether it is ahead of the
lockfile. Exits with code `0` when everything is up to date and code
`1` when at least one dependency has updates available. Suitable for
CI: run `skem check` to fail the build when the lockfile is stale.

### `skem self-update`

Update the `skem` binary itself to the latest release. Network access
to GitHub is required.

### `skem llms`

Print this guide to stdout. Intended for LLM agents and for piping
into agent context files.

## Configuration file format (`.skem.yaml`)

```yaml
deps:
  - name: example-api
    repo: "https://github.com/example/api.git"
    rev: "main"                 # optional; defaults to remote HEAD
    paths:
      - "proto/v1/"             # directory (trailing slash) or file
      - "schemas/user.proto"    # individual files are supported
    out: "./vendor/api"
    hooks:
      - "echo 'Files updated'"
post_hooks:
  - "echo 'All dependencies synced'"
```

### Fields

| Field            | Required | Description                                                                  |
| ---------------- | -------- | ---------------------------------------------------------------------------- |
| `deps`           | yes      | List of dependencies.                                                        |
| `deps[].name`    | yes      | Identifier; also used as the lockfile key.                                   |
| `deps[].repo`    | yes      | Git repository URL (any URL `git` can clone).                                |
| `deps[].rev`     | no       | Branch, tag, or commit SHA. Defaults to the remote's default branch (HEAD). |
| `deps[].paths`   | yes      | List of paths inside the repo to fetch via sparse-checkout.                  |
| `deps[].out`     | yes      | Local output directory; created if absent.                                   |
| `deps[].hooks`   | no       | Commands run after that single dependency is synced (only on change).        |
| `post_hooks`     | no       | Commands run once after all dependencies finish syncing.                     |

### Path semantics

- Paths ending with `/` are directories; their *contents* are copied
  into `out`, preserving the directory's internal structure but
  stripping the leading path prefix.
- Paths without `/` may be files; the file is copied directly into
  `out`.
- Multiple `paths` entries are merged into the same `out` directory.

### Hooks

`hooks` and `post_hooks` are executed via the user's shell. They run
in the project root (the directory containing `.skem.yaml`).

Per-dependency `hooks` receive the following environment variable:

- `SKEM_SYNCED_FILES` — space-separated list of file paths (under
  `out`) that were written during the sync. Example:

  ```yaml
  hooks:
    - "protoc --go_out=. $SKEM_SYNCED_FILES"
  ```

`post_hooks` do not receive `SKEM_SYNCED_FILES`.

## Lockfile (`.skem.lock`)

`.skem.lock` is YAML and looks like:

```yaml
locks:
  - name: example-api
    repo: "https://github.com/example/api.git"
    rev: "main"
    sha: "0123456789abcdef0123456789abcdef01234567"
```

Treat it as machine-managed: do not hand-edit it. Commit it to VCS so
that other developers and CI get reproducible syncs.

## Typical workflows for an agent

- **First time in a repo without skem**:
  `skem init` → edit `.skem.yaml` or use `skem add` → `skem sync`.
- **Bumping a dependency to a new revision**: edit its `rev` in
  `.skem.yaml`, then `skem sync`.
- **Restoring vendored files after local edits**: `skem sync --force`.
- **Re-running a hook after editing the hook command**:
  `skem sync --hooks-only`.
- **CI gate that vendored files are current**: `skem check` (non-zero
  exit means the lockfile is behind a remote).
- **Auditing the schema in an editor**: `skem schema > schema.json`
  and point the editor at it.

## Exit codes

- `0` — success (and, for `skem check`, "everything up to date").
- `1` — any error, or for `skem check`, "updates are available".
