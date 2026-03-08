use clap::{Parser, Subcommand};
use skem::{change_detection, config, init, schema, sync};
use std::fs;
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
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Some(Commands::Init) => init::init(),
        Some(Commands::Schema) => schema::schema(),
        Some(Commands::Sync) | None => run_sync(),
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn run_sync() -> anyhow::Result<()> {
    // Read configuration file
    let config_path = Path::new(".skem.yaml");
    if !config_path.exists() {
        anyhow::bail!(
            ".skem.yaml not found. Run 'skem init' to create a sample configuration file."
        );
    }

    let config_content = fs::read_to_string(config_path)?;
    let config: config::Config = serde_yaml::from_str(&config_content)?;

    if config.deps.is_empty() {
        println!("No dependencies to synchronize.");
        return Ok(());
    }

    println!("Synchronizing {} dependencies...", config.deps.len());

    // Read existing lockfile
    let lockfile_path = Path::new(".skem.lock");
    let lockfile = change_detection::read_lockfile(lockfile_path)?;

    // Run synchronization in async context
    let rt = tokio::runtime::Runtime::new()?;
    let sync_results = rt.block_on(async { sync::sync_dependencies(&config).await })?;

    println!(
        "Successfully synchronized {} dependencies.",
        sync_results.len()
    );

    // Update lockfile with new SHAs
    let mut updated_lockfile = lockfile;
    change_detection::update_lockfile_entries(
        &mut updated_lockfile,
        sync_results
            .iter()
            .map(|(name, sha)| (name.as_str(), sha.as_str())),
    );

    // Write updated lockfile
    change_detection::write_lockfile(lockfile_path, &updated_lockfile)?;
    println!("Lockfile updated: {}", lockfile_path.display());

    Ok(())
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
}
