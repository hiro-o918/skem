use crate::config::{self, Config, Dependency, Lockfile};
use crate::copy::copy_files;
use crate::fetch::fetch_files;
use crate::git::GitCommand;
use crate::hooks::execute_hooks;
use crate::lockfile;
use anyhow::Result;
use std::path::Path;

/// Synchronize a single dependency
///
/// This function:
/// 1. Gets the latest commit SHA from git ls-remote
/// 2. Checks if the dependency has changed since the last sync
/// 3. If changed: fetches files, copies to output, and executes hooks
/// 4. If unchanged: skips synchronization
///
/// # Arguments
/// * `dependency` - The dependency to synchronize
/// * `current_lockfile` - Current lockfile to check for changes
///
/// # Returns
/// Some((name, repo, rev, sha)) if synced, None if skipped
pub fn sync_single_dependency(
    dependency: &Dependency,
    current_lockfile: &Lockfile,
) -> Result<Option<(String, String, String, String)>> {
    // Get latest SHA from remote (default to HEAD when rev is omitted)
    let rev = dependency.rev.as_deref().unwrap_or("HEAD");
    let sha = GitCommand::ls_remote(&dependency.repo, rev)?;

    // Skip if unchanged
    if !lockfile::has_changed(&dependency.name, &sha, current_lockfile) {
        println!("  {} is up to date, skipping.", dependency.name);
        return Ok(None);
    }

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

    Ok(Some((
        dependency.name.clone(),
        dependency.repo.clone(),
        rev.to_string(),
        sha,
    )))
}

/// Synchronize all dependencies in parallel
///
/// This function spawns a thread for each dependency and waits for all to complete.
/// If any dependency fails, the entire operation fails with the first error.
///
/// # Arguments
/// * `config` - Configuration containing all dependencies
/// * `current_lockfile` - Current lockfile to check for changes
///
/// # Returns
/// Vector of (name, repo, rev, sha) tuples for synchronized (changed) dependencies
pub fn sync_dependencies(
    config: &Config,
    current_lockfile: &Lockfile,
) -> Result<Vec<(String, String, String, String)>> {
    let handles: Vec<_> = config
        .deps
        .iter()
        .map(|dep| {
            let dep = dep.clone();
            let lf = current_lockfile.clone();
            std::thread::spawn(move || sync_single_dependency(&dep, &lf))
        })
        .collect();

    let results: Vec<Option<(String, String, String, String)>> = handles
        .into_iter()
        .map(|h| h.join().map_err(|_| anyhow::anyhow!("Thread panicked"))?)
        .collect::<Result<_>>()?;

    Ok(results.into_iter().flatten().collect())
}

/// Run the full synchronization workflow
///
/// This function:
/// 1. Reads and validates the .skem.yaml configuration
/// 2. Reads the existing lockfile
/// 3. Executes parallel synchronization of all dependencies (skipping unchanged)
/// 4. Updates and writes the lockfile
pub fn run_sync() -> Result<()> {
    let config_path = Path::new(config::CONFIG_PATH);
    let config = config::read_config(config_path)?;

    if config.deps.is_empty() {
        println!("No dependencies to synchronize.");
        return Ok(());
    }

    println!("Synchronizing {} dependencies...", config.deps.len());

    let lockfile_path = Path::new(config::LOCKFILE_PATH);
    let current_lockfile = lockfile::read_lockfile(lockfile_path)?;

    let sync_results = sync_dependencies(&config, &current_lockfile)?;

    if sync_results.is_empty() {
        println!("All dependencies are up to date.");
        return Ok(());
    }

    println!(
        "Successfully synchronized {} dependencies.",
        sync_results.len()
    );

    let updated_lockfile = lockfile::update_lockfile_entries(
        &current_lockfile,
        sync_results.iter().map(|(name, repo, rev, sha)| {
            (name.as_str(), repo.as_str(), rev.as_str(), sha.as_str())
        }),
    );

    lockfile::write_lockfile(lockfile_path, &updated_lockfile)?;
    println!("Lockfile updated: {}", lockfile_path.display());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_dependencies_empty_config() {
        // Arrange: Empty config and lockfile
        let config = Config {
            deps: vec![],
            post_hooks: vec![],
        };
        let lockfile = Lockfile { locks: vec![] };

        // Act: Synchronize dependencies
        let result = sync_dependencies(&config, &lockfile);

        // Assert: Should succeed with empty results
        let synced = result.unwrap();
        let expected: Vec<(String, String, String, String)> = vec![];
        assert_eq!(synced, expected);
    }
}
