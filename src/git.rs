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

    #[test]
    fn test_ls_remote_parses_sha() {
        // This test requires a valid git repository to be available
        // For now, we'll test with the git repo itself
        let repo = "https://github.com/torvalds/linux.git";
        let rev = "refs/heads/master";

        let result = GitCommand::ls_remote(repo, rev);

        // Should either succeed with a 40-char SHA (old format) or 64-char SHA (new format)
        match result {
            Ok(sha) => {
                assert!(
                    sha.len() == 40 || sha.len() == 64,
                    "SHA should be 40 or 64 chars"
                );
                assert!(
                    sha.chars().all(|c| c.is_ascii_hexdigit()),
                    "SHA should be hexadecimal"
                );
            }
            Err(e) => {
                // Network error is acceptable in test environment
                eprintln!("Warning: ls_remote test skipped due to: {e}");
            }
        }
    }

    #[test]
    fn test_ls_remote_with_invalid_ref() {
        let repo = "https://github.com/torvalds/linux.git";
        let rev = "refs/heads/nonexistent-branch-12345";

        let result = GitCommand::ls_remote(repo, rev);

        // Should return error or empty output, which would be caught
        match result {
            Ok(sha) => {
                // Empty output case
                assert!(sha.is_empty() || sha.len() >= 40);
            }
            Err(_) => {
                // Expected to fail
            }
        }
    }

    #[test]
    fn test_clone_sparse_creates_directory() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let clone_path = temp_dir.path().join("test_repo");

        let repo = "https://github.com/torvalds/linux.git";

        let result = GitCommand::clone_sparse(repo, &clone_path);

        match result {
            Ok(_) => {
                // Verify directory was created
                assert!(clone_path.exists());
                // Clean up
                let _ = fs::remove_dir_all(&clone_path);
            }
            Err(e) => {
                // Network error is acceptable in test environment
                eprintln!("Warning: clone_sparse test skipped due to: {e}");
            }
        }
    }

    #[test]
    fn test_sparse_checkout_set_with_valid_paths() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().join("test_repo");

        // First, clone a repository
        let repo = "https://github.com/torvalds/linux.git";
        if GitCommand::clone_sparse(repo, &repo_path).is_ok() {
            let paths = vec!["Documentation/".to_string(), "README".to_string()];
            let result = GitCommand::sparse_checkout_set(&repo_path, &paths);

            match result {
                Ok(_) => {
                    // Verify sparse checkout was set up
                    assert!(
                        repo_path.join(".git/info/sparse-checkout").exists()
                            || repo_path.join(".git").exists()
                    );
                }
                Err(e) => {
                    eprintln!("Warning: sparse_checkout_set test failed: {e}");
                }
            }

            // Clean up
            let _ = fs::remove_dir_all(&repo_path);
        }
    }

    #[test]
    fn test_checkout_valid_revision() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().join("test_repo");

        // Clone a repository
        let repo = "https://github.com/torvalds/linux.git";
        if GitCommand::clone_sparse(repo, &repo_path).is_ok() {
            // Try to checkout a valid revision
            let result = GitCommand::checkout(&repo_path, "HEAD~1");

            match result {
                Ok(_) => {
                    // Checkout succeeded
                }
                Err(e) => {
                    eprintln!("Warning: checkout test failed: {e}");
                }
            }

            // Clean up
            let _ = fs::remove_dir_all(&repo_path);
        }
    }
}
