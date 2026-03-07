use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "skem")]
#[command(version, about = "A lightweight CLI tool to download specific files from remote Git repositories", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new .skem.yaml configuration file
    Init,
    /// Output JSON Schema for .skem.yaml configuration
    Schema,
    /// Synchronize dependencies (default command)
    Sync,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Init) => {
            println!("Running init command (stub)");
        }
        Some(Commands::Schema) => {
            println!("Running schema command (stub)");
        }
        Some(Commands::Sync) | None => {
            // デフォルトコマンドとして sync を実行
            println!("Running sync command (stub)");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn test_cli_verify() {
        // CLI定義の検証
        Cli::command().debug_assert();
    }

    #[test]
    fn test_default_is_sync() {
        // 引数なしの場合はSyncコマンドが実行されることを確認
        let cli = Cli::parse_from(vec!["skem"]);
        assert!(matches!(cli.command, None));
    }

    #[test]
    fn test_init_command_parsing() {
        // initコマンドのパース確認
        let cli = Cli::parse_from(vec!["skem", "init"]);
        assert!(matches!(cli.command, Some(Commands::Init)));
    }

    #[test]
    fn test_schema_command_parsing() {
        // schemaコマンドのパース確認
        let cli = Cli::parse_from(vec!["skem", "schema"]);
        assert!(matches!(cli.command, Some(Commands::Schema)));
    }

    #[test]
    fn test_sync_command_parsing() {
        // syncコマンドのパース確認
        let cli = Cli::parse_from(vec!["skem", "sync"]);
        assert!(matches!(cli.command, Some(Commands::Sync)));
    }
}
