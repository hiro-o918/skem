# Commands

## `skem init`

Creates a `.skem.yaml` file in the current directory with an example configuration.

```bash
skem init
```

## `skem sync`

Downloads files from configured repositories and runs hooks if there are changes.

```bash
skem sync
```

## `skem add`

Adds a new dependency to the configuration.

```bash
skem add --repo https://github.com/example/api.git --paths proto/ --out ./vendor/api
```

You can also specify an optional name and revision:

```bash
skem add --repo https://github.com/example/api.git --paths proto/ openapi/ --out ./vendor/api --name my-api --rev v2.0
```

### Interactive mode

Omit `--paths` and `--out` to enter interactive mode, which lets you browse the repository and select files:

```bash
skem add --repo https://github.com/example/api.git
```

## `skem rm`

Removes a dependency from both `.skem.yaml` and `.skem.lock`.

```bash
skem rm <name>
```

## `skem ls`

Lists all configured dependencies with their sync status.

```bash
skem ls
```

## `skem check`

Checks if any dependencies have updates available compared to the lockfile. Exits with code 1 if updates are found.

```bash
skem check
```

## `skem schema`

Outputs JSON Schema for `.skem.yaml` configuration file, useful for editor completion.

```bash
skem schema
```
