use anyhow::Result;
use tempfile::TempDir;

use crate::config::Dependency;
use crate::git::GitCommand;

/// Fetch files from a Git repository using sparse checkout
///
/// # Arguments
/// * `dep` - Dependency configuration
/// * `commit_sha` - Specific commit SHA to checkout
///
/// # Returns
/// A TempDir containing the fetched files. The directory will be automatically
/// deleted when the TempDir is dropped.
pub fn fetch_files(dep: &Dependency, commit_sha: &str) -> Result<TempDir> {
    // Create a temporary directory
    let temp_dir =
        TempDir::new().map_err(|e| anyhow::anyhow!("Failed to create temporary directory: {e}"))?;
    let repo_path = temp_dir.path();

    // Clone the repository with sparse checkout enabled
    GitCommand::clone_sparse(&dep.repo, repo_path)
        .map_err(|e| anyhow::anyhow!("Failed to clone repository '{}': {e}", dep.repo))?;

    // Set sparse checkout paths
    GitCommand::sparse_checkout_set(repo_path, &dep.paths)
        .map_err(|e| anyhow::anyhow!("Failed to set sparse checkout paths {:?}: {e}", dep.paths))?;

    // Checkout the specific commit
    GitCommand::checkout(repo_path, commit_sha)
        .map_err(|e| anyhow::anyhow!("Failed to checkout commit '{commit_sha}': {e}"))?;

    Ok(temp_dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Repository URL for testing
    const TEST_REPO: &str = "https://github.com/hiro-o918/skem.git";

    #[test]
    fn test_fetch_files_creates_temp_directory() {
        // Arrange: Create a test dependency
        let dep = Dependency {
            name: "test-dep".to_string(),
            repo: TEST_REPO.to_string(),
            rev: "main".to_string(),
            paths: vec!["src/".to_string()],
            out: "./vendor/test".to_string(),
            hooks: vec![],
        };

        // Get the actual commit SHA for main branch
        let commit_sha =
            GitCommand::ls_remote(TEST_REPO, "refs/heads/main").expect("Should get commit SHA");

        // Act: Fetch files
        let temp_dir = fetch_files(&dep, &commit_sha);

        // Assert: TempDir should be created successfully
        assert!(temp_dir.is_ok(), "fetch_files should succeed: {temp_dir:?}");
        let temp_dir = temp_dir.unwrap();
        assert!(temp_dir.path().exists(), "Temporary directory should exist");
        assert!(
            temp_dir.path().join(".git").exists(),
            ".git directory should exist"
        );
    }

    #[test]
    fn test_fetch_files_checks_out_specified_paths() {
        // Arrange: Create a test dependency with specific paths (directories only)
        let dep = Dependency {
            name: "test-dep".to_string(),
            repo: TEST_REPO.to_string(),
            rev: "main".to_string(),
            paths: vec!["src/".to_string(), ".github/".to_string()],
            out: "./vendor/test".to_string(),
            hooks: vec![],
        };

        // Get the actual commit SHA for main branch
        let commit_sha =
            GitCommand::ls_remote(TEST_REPO, "refs/heads/main").expect("Should get commit SHA");

        // Act: Fetch files
        let temp_dir = fetch_files(&dep, &commit_sha).expect("Should fetch files");

        // Assert: Specified paths should exist
        assert!(
            temp_dir.path().join("src").exists(),
            "src/ directory should be checked out"
        );
        assert!(
            temp_dir.path().join(".github").exists(),
            ".github/ directory should be checked out"
        );
    }

    #[test]
    fn test_fetch_files_checks_out_specific_commit() {
        // Arrange: Create a test dependency with directory path
        let dep = Dependency {
            name: "test-dep".to_string(),
            repo: TEST_REPO.to_string(),
            rev: "main".to_string(),
            paths: vec!["src/".to_string()],
            out: "./vendor/test".to_string(),
            hooks: vec![],
        };

        // Get a specific commit SHA (use main for testing)
        let commit_sha =
            GitCommand::ls_remote(TEST_REPO, "refs/heads/main").expect("Should get commit SHA");

        // Act: Fetch files
        let temp_dir = fetch_files(&dep, &commit_sha).expect("Should fetch files");

        // Assert: Should checkout the specific commit
        // We verify this by checking that the directory exists and contains files
        assert!(
            temp_dir.path().join("src").exists(),
            "src/ directory should be checked out"
        );
        // Verify that src directory contains Rust files
        assert!(
            temp_dir.path().join("src").join("main.rs").exists()
                || temp_dir.path().join("src").join("lib.rs").exists(),
            "src/ should contain Rust source files"
        );
    }

    #[test]
    fn test_fetch_files_temp_dir_auto_deleted() {
        // Arrange: Create a test dependency with directory path
        let dep = Dependency {
            name: "test-dep".to_string(),
            repo: TEST_REPO.to_string(),
            rev: "main".to_string(),
            paths: vec!["src/".to_string()],
            out: "./vendor/test".to_string(),
            hooks: vec![],
        };

        let commit_sha =
            GitCommand::ls_remote(TEST_REPO, "refs/heads/main").expect("Should get commit SHA");

        // Act: Fetch files and capture the path
        let temp_path = {
            let temp_dir = fetch_files(&dep, &commit_sha).expect("Should fetch files");
            let path = temp_dir.path().to_path_buf();
            assert!(path.exists(), "Path should exist while TempDir is in scope");
            path
            // temp_dir is dropped here
        };

        // Assert: Directory should be automatically deleted after TempDir is dropped
        assert!(
            !temp_path.exists(),
            "Temporary directory should be automatically deleted"
        );
    }
}
