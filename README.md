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

### Install script (recommended)

```bash
curl -fsSL https://raw.githubusercontent.com/hiro-o918/skem/main/install.sh | bash
```

You can also specify a version or install directory:

```bash
curl -fsSL https://raw.githubusercontent.com/hiro-o918/skem/main/install.sh | VERSION=v0.1.0 bash
curl -fsSL https://raw.githubusercontent.com/hiro-o918/skem/main/install.sh | INSTALL_DIR=~/.local/bin bash
```

### From source

```bash
cargo install skem
```

## Quick Start

```bash
# Initialize configuration
skem init

# Add a dependency (interactive mode — browse and select files interactively)
skem add --repo https://github.com/example/api.git

# Or specify all options directly
skem add --repo https://github.com/example/api.git --paths proto/ --out ./vendor/api

# Synchronize dependencies
skem sync
```

## Documentation

- [Commands](docs/commands.md) - Full list of available commands
- [Configuration](docs/configuration.md) - Configuration file format and hooks
- [How It Works](docs/how-it-works.md) - Internal mechanism overview
- [Examples](examples/) - Example configurations

## License

MIT

## Author

Hironori Yamamoto
