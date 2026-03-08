use crate::config::{self, Config, Dependency, Lockfile};
use crate::copy::copy_files;
use crate::fetch::fetch_files;
use crate::git::GitCommand;
use crate::hooks::{execute_hooks, execute_hooks_with_env};
use crate::lockfile;
use crate::validate::validate_config;
use anyhow::Result;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

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

/// Collect all files recursively from a directory
///
/// Returns an empty vec if the directory does not exist.
fn collect_existing_files(out_dir: &Path) -> Result<Vec<PathBuf>> {
    if !out_dir.exists() {
        return Ok(vec![]);
    }
    let files = WalkDir::new(out_dir)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .map(|entry| entry.into_path())
        .collect();
    Ok(files)
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
/// * `force` - If true, skip the SHA change check and always sync
///
/// # Returns
/// Some((name, repo, rev, sha)) if synced, None if skipped
pub fn sync_single_dependency(
    dependency: &Dependency,
    current_lockfile: &Lockfile,
    force: bool,
) -> Result<Option<(String, String, String, String)>> {
    // Get latest SHA from remote (default to HEAD when rev is omitted)
    let rev = dependency.rev.as_deref().unwrap_or("HEAD");
    let sha = GitCommand::ls_remote(&dependency.repo, rev)?;

    // Skip if unchanged (unless force is enabled)
    if !force && !lockfile::has_changed(&dependency.name, &sha, current_lockfile) {
        println!("  {} is up to date, skipping.", dependency.name);
        return Ok(None);
    }

    // Fetch files from remote repository
    let temp_dir = fetch_files(dependency, &sha)?;

    // Copy files to output directory
    let copied_files = copy_files(
        temp_dir.path(),
        &dependency.paths,
        Path::new(&dependency.out),
    )?;
    validate_copied_count(copied_files.len(), &dependency.name, &dependency.paths)?;

    // Execute hooks if configured
    if !dependency.hooks.is_empty() {
        let synced_files_str = copied_files
            .iter()
            .map(|p| p.to_string_lossy())
            .collect::<Vec<_>>()
            .join(" ");
        let env_vars = vec![("SKEM_SYNCED_FILES", synced_files_str.as_str())];
        execute_hooks_with_env(&dependency.hooks, &env_vars)?;
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
/// * `force` - If true, skip the SHA change check and always sync
///
/// # Returns
/// Vector of (name, repo, rev, sha) tuples for synchronized (changed) dependencies
pub fn sync_dependencies(
    config: &Config,
    current_lockfile: &Lockfile,
    force: bool,
) -> Result<Vec<(String, String, String, String)>> {
    let handles: Vec<_> = config
        .deps
        .iter()
        .map(|dep| {
            let dep = dep.clone();
            let lf = current_lockfile.clone();
            std::thread::spawn(move || sync_single_dependency(&dep, &lf, force))
        })
        .collect();

    let results: Vec<Option<(String, String, String, String)>> = handles
        .into_iter()
        .map(|h| h.join().map_err(|_| anyhow::anyhow!("Thread panicked"))?)
        .collect::<Result<_>>()?;

    Ok(results.into_iter().flatten().collect())
}

/// Execute post hooks after all dependencies are synced
///
/// # Arguments
/// * `post_hooks` - List of shell commands to execute
///
/// # Returns
/// Result that succeeds if all hooks execute successfully
pub fn execute_post_hooks(post_hooks: &[String]) -> Result<()> {
    if post_hooks.is_empty() {
        return Ok(());
    }
    println!("Executing post hooks...");
    execute_hooks(post_hooks)
}

/// Run hooks only for all dependencies without fetching files
///
/// Collects existing files from each dependency's output directory
/// and executes hooks with SKEM_SYNCED_FILES set to those files.
fn run_hooks_only(config: &Config) -> Result<()> {
    println!("Running hooks only (skipping file sync)...");

    for dep in &config.deps {
        let out_dir = Path::new(&dep.out);
        let existing_files = collect_existing_files(out_dir)?;

        if !dep.hooks.is_empty() {
            println!("  Running hooks for {}...", dep.name);
            let synced_files_str = existing_files
                .iter()
                .map(|p| p.to_string_lossy())
                .collect::<Vec<_>>()
                .join(" ");
            let env_vars = vec![("SKEM_SYNCED_FILES", synced_files_str.as_str())];
            execute_hooks_with_env(&dep.hooks, &env_vars)?;
        }
    }

    execute_post_hooks(&config.post_hooks)?;

    println!("Hooks execution completed.");
    Ok(())
}

/// Run the full synchronization workflow
///
/// This function:
/// 1. Reads and validates the .skem.yaml configuration
/// 2. Reads the existing lockfile
/// 3. Executes parallel synchronization of all dependencies (skipping unchanged)
/// 4. Updates and writes the lockfile
pub fn run_sync(force: bool, hooks_only: bool) -> Result<()> {
    if force && hooks_only {
        anyhow::bail!("--force and --hooks-only cannot be used together");
    }

    let config_path = Path::new(config::CONFIG_PATH);
    let config = config::read_config(config_path)?;
    validate_config(&config)?;

    if config.deps.is_empty() {
        println!("No dependencies to synchronize.");
        return Ok(());
    }

    if hooks_only {
        return run_hooks_only(&config);
    }

    println!("Synchronizing {} dependencies...", config.deps.len());

    let lockfile_path = Path::new(config::LOCKFILE_PATH);
    let current_lockfile = lockfile::read_lockfile(lockfile_path)?;

    let sync_results = sync_dependencies(&config, &current_lockfile, force)?;

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

    // Execute post_hooks after all dependencies are synced
    execute_post_hooks(&config.post_hooks)?;

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
        let result = sync_dependencies(&config, &lockfile, false);

        // Assert: Should succeed with empty results
        let synced = result.unwrap();
        let expected: Vec<(String, String, String, String)> = vec![];
        assert_eq!(synced, expected);
    }

    #[test]
    fn test_execute_post_hooks_runs_hooks() {
        // Arrange: post_hooks that write to a temp file
        let temp_dir = tempfile::TempDir::new().unwrap();
        let output_path = temp_dir.path().join("post_hook_output.txt");
        let output_path_str = output_path.to_str().unwrap();
        let post_hooks = vec![format!("echo 'post hook executed' > {output_path_str}")];

        // Act: Execute post hooks
        let result = execute_post_hooks(&post_hooks);

        // Assert: Hook should have been executed
        assert!(result.is_ok());
        let content = std::fs::read_to_string(&output_path).unwrap();
        assert_eq!(content.trim(), "post hook executed");
    }

    #[test]
    fn test_execute_post_hooks_empty_list() {
        // Arrange: Empty post_hooks list
        let post_hooks: Vec<String> = vec![];

        // Act: Execute post hooks (should be a no-op)
        let result = execute_post_hooks(&post_hooks);

        // Assert: Should succeed without doing anything
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_post_hooks_fails_on_error() {
        // Arrange: post_hook that fails
        let post_hooks = vec!["exit 1".to_string()];

        // Act: Execute post hooks
        let result = execute_post_hooks(&post_hooks);

        // Assert: Should propagate the error
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_post_hooks_runs_multiple_hooks_in_order() {
        // Arrange: Multiple post_hooks that append to a file
        let temp_dir = tempfile::TempDir::new().unwrap();
        let output_path = temp_dir.path().join("post_hook_order.txt");
        let output_path_str = output_path.to_str().unwrap();
        let post_hooks = vec![
            format!("echo 'first' > {output_path_str}"),
            format!("echo 'second' >> {output_path_str}"),
        ];

        // Act: Execute post hooks
        let result = execute_post_hooks(&post_hooks);

        // Assert: Both hooks should have executed in order
        assert!(result.is_ok());
        let content = std::fs::read_to_string(&output_path).unwrap();
        assert_eq!(content.trim(), "first\nsecond");
    }

    #[test]
    fn test_run_hooks_only_executes_hooks_with_existing_files() {
        // Arrange: Create temp dirs for output with files and a hook that captures env
        let temp_dir = tempfile::TempDir::new().unwrap();
        let out_dir = temp_dir.path().join("out");
        std::fs::create_dir_all(&out_dir).unwrap();
        std::fs::write(out_dir.join("file1.txt"), "content1").unwrap();
        std::fs::write(out_dir.join("file2.txt"), "content2").unwrap();

        let hook_output_path = temp_dir.path().join("hook_output.txt");
        let hook_output_str = hook_output_path.to_str().unwrap();

        let config = Config {
            deps: vec![Dependency {
                name: "test-dep".to_string(),
                repo: "https://github.com/test/repo.git".to_string(),
                rev: Some("main".to_string()),
                paths: vec!["src/".to_string()],
                out: out_dir.to_str().unwrap().to_string(),
                hooks: vec![format!("echo $SKEM_SYNCED_FILES > {hook_output_str}")],
            }],
            post_hooks: vec![],
        };

        // Act: Run hooks only
        let result = run_hooks_only(&config);

        // Assert: Hook should have been executed with SKEM_SYNCED_FILES containing existing files
        assert!(result.is_ok());
        let content = std::fs::read_to_string(&hook_output_path).unwrap();
        let content = content.trim();
        // Files should be listed in SKEM_SYNCED_FILES
        assert!(content.contains("file1.txt"));
        assert!(content.contains("file2.txt"));
    }

    #[test]
    fn test_run_hooks_only_skips_deps_without_hooks() {
        // Arrange: Config with a dependency that has no hooks
        let temp_dir = tempfile::TempDir::new().unwrap();
        let out_dir = temp_dir.path().join("out");
        std::fs::create_dir_all(&out_dir).unwrap();

        let config = Config {
            deps: vec![Dependency {
                name: "test-dep".to_string(),
                repo: "https://github.com/test/repo.git".to_string(),
                rev: Some("main".to_string()),
                paths: vec!["src/".to_string()],
                out: out_dir.to_str().unwrap().to_string(),
                hooks: vec![],
            }],
            post_hooks: vec![],
        };

        // Act: Run hooks only (should succeed without executing any hooks)
        let result = run_hooks_only(&config);

        // Assert: Should succeed
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_hooks_only_executes_post_hooks() {
        // Arrange: Config with post_hooks
        let temp_dir = tempfile::TempDir::new().unwrap();
        let out_dir = temp_dir.path().join("out");
        std::fs::create_dir_all(&out_dir).unwrap();

        let post_hook_output = temp_dir.path().join("post_hook_output.txt");
        let post_hook_output_str = post_hook_output.to_str().unwrap();

        let config = Config {
            deps: vec![Dependency {
                name: "test-dep".to_string(),
                repo: "https://github.com/test/repo.git".to_string(),
                rev: Some("main".to_string()),
                paths: vec!["src/".to_string()],
                out: out_dir.to_str().unwrap().to_string(),
                hooks: vec![],
            }],
            post_hooks: vec![format!("echo 'post hook ran' > {post_hook_output_str}")],
        };

        // Act: Run hooks only
        let result = run_hooks_only(&config);

        // Assert: Post hook should have been executed
        assert!(result.is_ok());
        let content = std::fs::read_to_string(&post_hook_output).unwrap();
        assert_eq!(content.trim(), "post hook ran");
    }

    #[test]
    fn test_collect_existing_files() {
        // Arrange: Create a temp directory with some files
        let temp_dir = tempfile::TempDir::new().unwrap();
        let out_dir = temp_dir.path();
        std::fs::create_dir_all(out_dir.join("subdir")).unwrap();
        std::fs::write(out_dir.join("file1.txt"), "content1").unwrap();
        std::fs::write(out_dir.join("file2.txt"), "content2").unwrap();
        std::fs::write(out_dir.join("subdir").join("file3.txt"), "content3").unwrap();

        // Act: Collect existing files
        let mut files = collect_existing_files(out_dir).unwrap();
        files.sort();

        // Assert: All files should be collected
        let mut expected = vec![
            out_dir.join("file1.txt"),
            out_dir.join("file2.txt"),
            out_dir.join("subdir").join("file3.txt"),
        ];
        expected.sort();
        assert_eq!(files, expected);
    }

    #[test]
    fn test_collect_existing_files_empty_dir() {
        // Arrange: Create an empty temp directory
        let temp_dir = tempfile::TempDir::new().unwrap();
        let out_dir = temp_dir.path();

        // Act: Collect existing files
        let files = collect_existing_files(out_dir).unwrap();

        // Assert: Should return empty vec
        let expected: Vec<PathBuf> = vec![];
        assert_eq!(files, expected);
    }

    #[test]
    fn test_collect_existing_files_nonexistent_dir() {
        // Arrange: Non-existent directory path
        let out_dir = Path::new("/tmp/nonexistent_skem_test_dir_12345");

        // Act: Collect existing files
        let files = collect_existing_files(out_dir).unwrap();

        // Assert: Should return empty vec
        let expected: Vec<PathBuf> = vec![];
        assert_eq!(files, expected);
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
