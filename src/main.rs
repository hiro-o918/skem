use clap::{Parser, Subcommand};
use skem::{add, config, init, ls, rm, schema, sync};
use std::path::Path;

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
    /// Add a new dependency
    Add {
        /// Git repository URL
        #[arg(long)]
        repo: String,
        /// Paths to download from the repository
        #[arg(long, num_args = 1..)]
        paths: Vec<String>,
        /// Output directory
        #[arg(long)]
        out: String,
        /// Dependency name (defaults to repository name)
        #[arg(long)]
        name: Option<String>,
        /// Branch, tag, or commit hash (defaults to HEAD)
        #[arg(long)]
        rev: Option<String>,
    },
    /// Remove a dependency
    Rm {
        /// Name of the dependency to remove
        name: String,
    },
    /// List all dependencies
    Ls,
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Some(Commands::Init) => init::init(),
        Some(Commands::Schema) => schema::schema(),
        Some(Commands::Sync) | None => sync::run_sync(),
        Some(Commands::Add {
            repo,
            paths,
            out,
            name,
            rev,
        }) => add::run_add(
            Path::new(config::CONFIG_PATH),
            &repo,
            paths,
            &out,
            name.as_deref(),
            rev.as_deref(),
        ),
        Some(Commands::Rm { name }) => rm::run_rm_default(&name),
        Some(Commands::Ls) => ls::run_ls_default(),
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
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
        assert!(cli.command.is_none());
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

    #[test]
    fn test_add_command_parsing() {
        let cli = Cli::parse_from(vec![
            "skem",
            "add",
            "--repo",
            "https://github.com/example/api.git",
            "--paths",
            "proto/",
            "--out",
            "./vendor/api",
        ]);
        assert!(matches!(cli.command, Some(Commands::Add { .. })));
    }

    #[test]
    fn test_add_command_parsing_with_all_options() {
        let cli = Cli::parse_from(vec![
            "skem",
            "add",
            "--repo",
            "https://github.com/example/api.git",
            "--paths",
            "proto/",
            "openapi/",
            "--out",
            "./vendor/api",
            "--name",
            "my-api",
            "--rev",
            "v2.0",
        ]);
        match cli.command {
            Some(Commands::Add {
                repo,
                paths,
                out,
                name,
                rev,
            }) => {
                assert_eq!(repo, "https://github.com/example/api.git");
                assert_eq!(paths, vec!["proto/", "openapi/"]);
                assert_eq!(out, "./vendor/api");
                assert_eq!(name, Some("my-api".to_string()));
                assert_eq!(rev, Some("v2.0".to_string()));
            }
            _ => panic!("Expected Add command"),
        }
    }

    #[test]
    fn test_rm_command_parsing() {
        let cli = Cli::parse_from(vec!["skem", "rm", "my-dep"]);
        match cli.command {
            Some(Commands::Rm { name }) => {
                assert_eq!(name, "my-dep");
            }
            _ => panic!("Expected Rm command"),
        }
    }

    #[test]
    fn test_ls_command_parsing() {
        let cli = Cli::parse_from(vec!["skem", "ls"]);
        assert!(matches!(cli.command, Some(Commands::Ls)));
    }
}
