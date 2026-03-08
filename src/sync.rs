use crate::config::{Config, Dependency};
use crate::copy::copy_files;
use crate::fetch::fetch_files;
use crate::git::GitCommand;
use crate::hooks::execute_hooks;
use anyhow::Result;
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
/// This function spawns a tokio task for each dependency and waits for all to complete.
/// If any dependency fails, the entire operation fails with the first error.
///
/// # Arguments
/// * `config` - Configuration containing all dependencies
///
/// # Returns
/// Vector of (dependency_name, new_sha) tuples for successfully synchronized dependencies
pub async fn sync_dependencies(config: &Config) -> Result<Vec<(String, String)>> {
    let mut tasks = vec![];

    for dep in &config.deps {
        let dep = dep.clone();

        // Spawn blocking task for synchronization
        let task = tokio::task::spawn_blocking(move || sync_single_dependency(&dep));

        tasks.push(task);
    }

    // Wait for all tasks to complete and collect results
    let mut results = vec![];
    for task in tasks {
        match task.await {
            Ok(Ok((name, sha))) => results.push((name, sha)),
            Ok(Err(e)) => return Err(e),
            Err(e) => return Err(anyhow::anyhow!("Task join error: {e}")),
        }
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sync_dependencies_empty_config() {
        // Arrange: Empty config
        let config = Config { deps: vec![] };

        // Act: Synchronize dependencies
        let result = sync_dependencies(&config).await;

        // Assert: Should succeed with empty results
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }
}
