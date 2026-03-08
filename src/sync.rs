use crate::config::{self, Config, Dependency, Lockfile};
use crate::copy::copy_files;
use crate::fetch::fetch_files;
use crate::git::GitCommand;
use crate::hooks::execute_hooks;
use crate::lockfile;
use crate::validate::validate_config;
use anyhow::Result;
use std::path::Path;

/// Validate that copy_files copied at least one file
///
/// Returns an error if no files were matched, indicating a possible misconfiguration.
fn validate_copied_count(count: usize, dep_name: &str, dep_paths: &[String]) -> Result<()> {
    if count == 0 {
        anyhow::bail!(
            "No files matched paths {dep_paths:?} for dependency '{dep_name}'. Check your paths configuration.",
        );
    }
    Ok(())
}

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
    let copied_count = copy_files(
        temp_dir.path(),
        &dependency.paths,
        Path::new(&dependency.out),
    )?;
    validate_copied_count(copied_count, &dependency.name, &dependency.paths)?;

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
    validate_config(&config)?;

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

    #[test]
    fn test_validate_copied_count_with_files_copied() {
        // Arrange: 1 file was copied
        let dep_name = "my-dep";
        let dep_paths = vec!["proto/".to_string()];

        // Act: Validate with non-zero count
        let result = validate_copied_count(1, dep_name, &dep_paths);

        // Assert: Should succeed
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_copied_count_with_zero_files() {
        // Arrange: 0 files were copied
        let dep_name = "my-dep";
        let dep_paths = vec!["foo/".to_string()];

        // Act: Validate with zero count
        let result = validate_copied_count(0, dep_name, &dep_paths);

        // Assert: Should return an error with dependency name and paths
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("my-dep"),
            "Error should contain dependency name"
        );
        assert!(err_msg.contains("foo/"), "Error should contain the paths");
    }

    #[test]
    fn test_validate_copied_count_with_multiple_paths() {
        // Arrange: 0 files were copied with multiple paths
        let dep_name = "my-dep";
        let dep_paths = vec!["foo/".to_string(), "bar/".to_string()];

        // Act: Validate with zero count
        let result = validate_copied_count(0, dep_name, &dep_paths);

        // Assert: Should return an error containing all paths
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("foo/"));
        assert!(err_msg.contains("bar/"));
    }
}
