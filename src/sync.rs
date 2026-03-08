use crate::change_detection;
use crate::config::{Config, Dependency};
use crate::copy::copy_files;
use crate::fetch::fetch_files;
use crate::git::GitCommand;
use crate::hooks::execute_hooks;
use anyhow::Result;
use std::fs;
use std::path::Path;

/// Synchronize a single dependency
///
/// This function:
/// 1. Gets the latest commit SHA from git ls-remote
/// 2. Fetches files from the remote repository
/// 3. Copies files to the output directory
/// 4. Executes hooks if configured
///
/// # Arguments
/// * `dependency` - The dependency to synchronize
///
/// # Returns
/// Tuple of (dependency_name, new_sha) on success
pub fn sync_single_dependency(dependency: &Dependency) -> Result<(String, String)> {
    // Get latest SHA from remote
    let sha = GitCommand::ls_remote(&dependency.repo, &dependency.rev)?;

    // Fetch files from remote repository
    let temp_dir = fetch_files(dependency, &sha)?;

    // Copy files to output directory
    copy_files(
        temp_dir.path(),
        &dependency.paths,
        Path::new(&dependency.out),
    )?;

    // Execute hooks if configured
    if !dependency.hooks.is_empty() {
        execute_hooks(&dependency.hooks)?;
    }

    Ok((dependency.name.clone(), sha))
}

/// Synchronize all dependencies in parallel
///
/// This function spawns a thread for each dependency and waits for all to complete.
/// If any dependency fails, the entire operation fails with the first error.
///
/// # Arguments
/// * `config` - Configuration containing all dependencies
///
/// # Returns
/// Vector of (dependency_name, new_sha) tuples for successfully synchronized dependencies
pub fn sync_dependencies(config: &Config) -> Result<Vec<(String, String)>> {
    let handles: Vec<_> = config
        .deps
        .iter()
        .map(|dep| {
            let dep = dep.clone();
            std::thread::spawn(move || sync_single_dependency(&dep))
        })
        .collect();

    handles
        .into_iter()
        .map(|h| h.join().map_err(|_| anyhow::anyhow!("Thread panicked"))?)
        .collect()
}

/// Run the full synchronization workflow
///
/// This function:
/// 1. Reads and validates the .skem.yaml configuration
/// 2. Reads the existing lockfile
/// 3. Executes parallel synchronization of all dependencies
/// 4. Updates and writes the lockfile
pub fn run_sync() -> Result<()> {
    let config_path = Path::new(".skem.yaml");
    if !config_path.exists() {
        anyhow::bail!(
            ".skem.yaml not found. Run 'skem init' to create a sample configuration file."
        );
    }

    let config_content = fs::read_to_string(config_path)?;
    let config: Config = serde_yaml::from_str(&config_content)?;

    if config.deps.is_empty() {
        println!("No dependencies to synchronize.");
        return Ok(());
    }

    println!("Synchronizing {} dependencies...", config.deps.len());

    let lockfile_path = Path::new(".skem.lock");
    let lockfile = change_detection::read_lockfile(lockfile_path)?;

    let sync_results = sync_dependencies(&config)?;

    println!(
        "Successfully synchronized {} dependencies.",
        sync_results.len()
    );

    let updated_lockfile = change_detection::update_lockfile_entries(
        &lockfile,
        sync_results
            .iter()
            .map(|(name, sha)| (name.as_str(), sha.as_str())),
    );

    change_detection::write_lockfile(lockfile_path, &updated_lockfile)?;
    println!("Lockfile updated: {}", lockfile_path.display());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_dependencies_empty_config() {
        // Arrange: Empty config
        let config = Config { deps: vec![] };

        // Act: Synchronize dependencies
        let result = sync_dependencies(&config);

        // Assert: Should succeed with empty results
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }
}
