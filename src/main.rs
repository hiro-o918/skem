use clap::{Parser, Subcommand};
use skem::{add, check, config, init, interactive, ls, rm, schema, sync};
use std::path::Path;

#[derive(Parser)]
#[command(name = "skem")]
#[command(version, about = "A lightweight CLI tool to download specific files from remote Git repositories", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
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
        /// Paths to download from the repository (omit for interactive mode)
        #[arg(long, num_args = 1..)]
        paths: Option<Vec<String>>,
        /// Output directory (omit for interactive mode)
        #[arg(long)]
        out: Option<String>,
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
    /// Check if any dependencies have updates available
    Check,
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Init => init::init(),
        Commands::Schema => schema::schema(),
        Commands::Sync => sync::run_sync(),
        Commands::Add {
            repo,
            paths,
            out,
            name,
            rev,
        } => match (paths, out) {
            (Some(paths), Some(out)) => add::run_add(
                Path::new(config::CONFIG_PATH),
                &repo,
                paths,
                &out,
                name.as_deref(),
                rev.as_deref(),
            ),
            (None, None) => interactive::run_interactive_add(
                Path::new(config::CONFIG_PATH),
                &repo,
                rev.as_deref(),
                name.as_deref(),
            ),
            _ => {
                eprintln!("Error: --paths and --out must be specified together, or both omitted for interactive mode.");
                std::process::exit(1);
            }
        },
        Commands::Rm { name } => rm::run_rm_default(&name),
        Commands::Ls => ls::run_ls_default(),
        Commands::Check => match check::run_check_default() {
            Ok(true) => Ok(()),
            Ok(false) => std::process::exit(1),
            Err(e) => Err(e),
        },
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
    fn test_no_subcommand_shows_help() {
        // No subcommand should result in a parse error (help displayed)
        let result = Cli::try_parse_from(vec!["skem"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_init_command_parsing() {
        let cli = Cli::parse_from(vec!["skem", "init"]);
        assert!(matches!(cli.command, Commands::Init));
    }

    #[test]
    fn test_schema_command_parsing() {
        let cli = Cli::parse_from(vec!["skem", "schema"]);
        assert!(matches!(cli.command, Commands::Schema));
    }

    #[test]
    fn test_sync_command_parsing() {
        let cli = Cli::parse_from(vec!["skem", "sync"]);
        assert!(matches!(cli.command, Commands::Sync));
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
        assert!(matches!(cli.command, Commands::Add { .. }));
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
            Commands::Add {
                repo,
                paths,
                out,
                name,
                rev,
            } => {
                assert_eq!(repo, "https://github.com/example/api.git");
                assert_eq!(
                    paths,
                    Some(vec!["proto/".to_string(), "openapi/".to_string()])
                );
                assert_eq!(out, Some("./vendor/api".to_string()));
                assert_eq!(name, Some("my-api".to_string()));
                assert_eq!(rev, Some("v2.0".to_string()));
            }
            _ => panic!("Expected Add command"),
        }
    }

    #[test]
    fn test_add_command_parsing_interactive_mode() {
        // --paths and --out omitted for interactive mode
        let cli = Cli::parse_from(vec![
            "skem",
            "add",
            "--repo",
            "https://github.com/example/api.git",
        ]);
        match cli.command {
            Commands::Add {
                repo,
                paths,
                out,
                name,
                rev,
            } => {
                assert_eq!(repo, "https://github.com/example/api.git");
                assert_eq!(paths, None);
                assert_eq!(out, None);
                assert_eq!(name, None);
                assert_eq!(rev, None);
            }
            _ => panic!("Expected Add command"),
        }
    }

    #[test]
    fn test_rm_command_parsing() {
        let cli = Cli::parse_from(vec!["skem", "rm", "my-dep"]);
        match cli.command {
            Commands::Rm { name } => {
                assert_eq!(name, "my-dep");
            }
            _ => panic!("Expected Rm command"),
        }
    }

    #[test]
    fn test_ls_command_parsing() {
        let cli = Cli::parse_from(vec!["skem", "ls"]);
        assert!(matches!(cli.command, Commands::Ls));
    }

    #[test]
    fn test_check_command_parsing() {
        let cli = Cli::parse_from(vec!["skem", "check"]);
        assert!(matches!(cli.command, Commands::Check));
    }
}
