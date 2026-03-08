use anyhow::Result;
use std::path::Path;
use std::process::Command;

/// Wrapper for Git commands using std::process::Command
pub struct GitCommand;

impl GitCommand {
    /// Execute `git ls-remote <repo> <rev>` to get the commit SHA
    ///
    /// # Arguments
    /// * `repo` - Repository URL
    /// * `rev` - Revision (branch, tag, or commit)
    ///
    /// # Returns
    /// The commit SHA of the specified revision
    pub fn ls_remote(repo: &str, rev: &str) -> Result<String> {
        let output = Command::new("git")
            .arg("ls-remote")
            .arg(repo)
            .arg(rev)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("git ls-remote failed: {stderr}");
        }

        let stdout = String::from_utf8(output.stdout)?;
        let sha = stdout
            .split_whitespace()
            .next()
            .ok_or_else(|| anyhow::anyhow!("No output from git ls-remote"))?
            .to_string();

        Ok(sha)
    }

    /// Execute `git clone --depth 1 --filter=blob:none --sparse <repo> <path>`
    ///
    /// # Arguments
    /// * `repo` - Repository URL
    /// * `path` - Destination path
    pub fn clone_sparse(repo: &str, path: &Path) -> Result<()> {
        let output = Command::new("git")
            .arg("clone")
            .arg("--depth")
            .arg("1")
            .arg("--filter=blob:none")
            .arg("--sparse")
            .arg(repo)
            .arg(path)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("git clone failed: {stderr}");
        }

        Ok(())
    }

    /// Execute `git sparse-checkout set <paths...>` in the cloned repository
    ///
    /// # Arguments
    /// * `repo_path` - Path to the cloned repository
    /// * `paths` - Paths to checkout
    pub fn sparse_checkout_set(repo_path: &Path, paths: &[String]) -> Result<()> {
        let mut cmd = Command::new("git");
        cmd.arg("sparse-checkout").arg("set");

        for path in paths {
            cmd.arg(path);
        }

        let output = cmd.current_dir(repo_path).output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("git sparse-checkout set failed: {stderr}");
        }

        Ok(())
    }

    /// Execute `git clone --depth 1 --filter=blob:none --no-checkout <repo> <path>`
    ///
    /// Clones a repository without checking out any files,
    /// fetching only tree objects (no blobs).
    ///
    /// # Arguments
    /// * `repo` - Repository URL
    /// * `path` - Destination path
    pub fn clone_blobless(repo: &str, path: &Path) -> Result<()> {
        let output = Command::new("git")
            .arg("clone")
            .arg("--depth")
            .arg("1")
            .arg("--filter=blob:none")
            .arg("--no-checkout")
            .arg(repo)
            .arg(path)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("git clone (blobless) failed: {stderr}");
        }

        Ok(())
    }

    /// Execute `git ls-tree -r --name-only <rev>` to list files in the repository
    ///
    /// # Arguments
    /// * `repo_path` - Path to the cloned repository
    /// * `rev` - Revision to list files from
    ///
    /// # Returns
    /// A list of file paths in the repository
    pub fn ls_tree(repo_path: &Path, rev: &str) -> Result<Vec<String>> {
        let output = Command::new("git")
            .arg("ls-tree")
            .arg("-r")
            .arg("--name-only")
            .arg(rev)
            .current_dir(repo_path)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("git ls-tree failed: {stderr}");
        }

        let stdout = String::from_utf8(output.stdout)?;
        let paths: Vec<String> = stdout.lines().map(|l| l.to_string()).collect();

        Ok(paths)
    }

    /// Execute `git checkout <rev>` in the specified repository
    ///
    /// # Arguments
    /// * `repo_path` - Path to the repository
    /// * `rev` - Revision to checkout
    pub fn checkout(repo_path: &Path, rev: &str) -> Result<()> {
        let output = Command::new("git")
            .arg("checkout")
            .arg(rev)
            .current_dir(repo_path)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("git checkout failed: {stderr}");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// Repository URL for testing
    const TEST_REPO: &str = "https://github.com/hiro-o918/skem.git";

    #[test]
    fn test_ls_remote_parses_sha() {
        // Query main branch to get a valid SHA
        let result = GitCommand::ls_remote(TEST_REPO, "refs/heads/main");

        match result {
            Ok(sha) => {
                // Should be 40-char SHA (old format) or 64-char SHA (new format)
                assert!(
                    sha.len() == 40 || sha.len() == 64,
                    "SHA should be 40 or 64 chars, got: {sha}"
                );
                assert!(
                    sha.chars().all(|c| c.is_ascii_hexdigit()),
                    "SHA should be hexadecimal, got: {sha}"
                );
            }
            Err(e) => {
                panic!("ls_remote should succeed for this repository: {e}");
            }
        }
    }

    #[test]
    fn test_ls_remote_with_invalid_ref() {
        let result = GitCommand::ls_remote(TEST_REPO, "refs/heads/nonexistent-branch-xyz");

        // Should return error since the ref doesn't exist
        assert!(result.is_err(), "ls_remote should fail for nonexistent ref");
    }

    #[test]
    fn test_clone_sparse_creates_directory() {
        let temp_dir = TempDir::new().unwrap();
        let clone_path = temp_dir.path().join("test_repo");

        let result = GitCommand::clone_sparse(TEST_REPO, &clone_path);

        assert!(result.is_ok(), "clone_sparse should succeed: {result:?}");
        assert!(clone_path.exists(), "Cloned directory should exist");
        assert!(
            clone_path.join(".git").exists(),
            ".git directory should exist"
        );
    }

    #[test]
    fn test_sparse_checkout_set_with_valid_paths() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().join("test_repo");

        // Clone the repository
        let clone_result = GitCommand::clone_sparse(TEST_REPO, &repo_path);
        assert!(clone_result.is_ok(), "clone should succeed");

        // Set sparse checkout paths (must be directories)
        let paths = vec!["src/".to_string(), ".github/".to_string()];
        let result = GitCommand::sparse_checkout_set(&repo_path, &paths);

        assert!(
            result.is_ok(),
            "sparse_checkout_set should succeed: {result:?}"
        );
        assert!(
            repo_path.join(".git/info/sparse-checkout").exists(),
            "sparse-checkout file should exist"
        );
    }

    #[test]
    fn test_checkout_valid_revision() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().join("test_repo");

        // Clone the repository
        let clone_result = GitCommand::clone_sparse(TEST_REPO, &repo_path);
        assert!(clone_result.is_ok(), "clone should succeed");

        // Checkout a valid revision
        let result = GitCommand::checkout(&repo_path, "HEAD");

        assert!(result.is_ok(), "checkout HEAD should succeed: {result:?}");
    }

    #[test]
    fn test_clone_blobless_creates_git_directory() {
        let temp_dir = TempDir::new().unwrap();
        let clone_path = temp_dir.path().join("test_repo");

        let result = GitCommand::clone_blobless(TEST_REPO, &clone_path);

        assert!(result.is_ok(), "clone_blobless should succeed: {result:?}");
        assert!(clone_path.exists(), "Cloned directory should exist");
        assert!(
            clone_path.join(".git").exists(),
            ".git directory should exist"
        );
    }

    #[test]
    fn test_ls_tree_returns_file_paths() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().join("test_repo");

        GitCommand::clone_blobless(TEST_REPO, &repo_path).unwrap();

        let result = GitCommand::ls_tree(&repo_path, "HEAD");

        assert!(result.is_ok(), "ls_tree should succeed: {result:?}");
        let files = result.unwrap();
        assert!(!files.is_empty(), "Should return at least one file");
        assert!(
            files.iter().any(|f| f.contains("Cargo.toml")),
            "Should contain Cargo.toml"
        );
    }

    #[test]
    fn test_ls_tree_invalid_rev() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().join("test_repo");

        GitCommand::clone_blobless(TEST_REPO, &repo_path).unwrap();

        let result = GitCommand::ls_tree(&repo_path, "nonexistent-revision-xyz");

        assert!(result.is_err(), "ls_tree should fail for invalid revision");
    }

    #[test]
    fn test_checkout_invalid_revision() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().join("test_repo");

        // Clone the repository
        let clone_result = GitCommand::clone_sparse(TEST_REPO, &repo_path);
        assert!(clone_result.is_ok(), "clone should succeed");

        // Try to checkout an invalid revision
        let result = GitCommand::checkout(&repo_path, "nonexistent-revision-xyz");

        assert!(
            result.is_err(),
            "checkout should fail for nonexistent revision"
        );
    }
}
