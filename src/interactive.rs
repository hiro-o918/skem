use crate::add;
use crate::git::GitCommand;
use anyhow::Result;
use inquire::{MultiSelect, Text};
use std::collections::BTreeSet;
use std::path::Path;

/// Collect directory and file entries from a list of file paths.
///
/// Extracts all parent directories (with trailing `/`) and includes
/// original file paths. Results are sorted and deduplicated.
///
/// # Arguments
/// * `file_paths` - List of file paths from `git ls-tree`
///
/// # Returns
/// A sorted list of directory and file entries
pub fn collect_entries(file_paths: &[String]) -> Vec<String> {
    let mut entries = BTreeSet::new();

    for path in file_paths {
        entries.insert(path.clone());

        // Extract all parent directories
        let mut current = path.as_str();
        while let Some(pos) = current.rfind('/') {
            let dir = &current[..=pos];
            if !entries.insert(dir.to_string()) {
                // Already seen this directory, all parents are also already inserted
                break;
            }
            current = &current[..pos];
        }
    }

    entries.into_iter().collect()
}

/// List all entries (directories and files) in a remote repository.
///
/// Clones the repository in blobless mode and runs `ls-tree` to get file paths,
/// then expands them into directory + file entries.
///
/// # Arguments
/// * `repo` - Repository URL
/// * `rev` - Revision to list (e.g. "HEAD", "main", tag)
///
/// # Returns
/// A sorted list of directory and file entries
pub fn list_repo_entries(repo: &str, rev: &str) -> Result<Vec<String>> {
    let temp_dir = tempfile::tempdir()?;
    let repo_path = temp_dir.path().join("repo");

    eprintln!("Cloning repository...");
    GitCommand::clone_blobless(repo, &repo_path)?;

    eprintln!("Fetching file tree...");
    let file_paths = GitCommand::ls_tree(&repo_path, rev)?;

    Ok(collect_entries(&file_paths))
}

/// Prompt the user to select paths using fuzzy multi-select.
///
/// # Arguments
/// * `entries` - List of available entries (directories and files)
///
/// # Returns
/// The selected entries
pub fn prompt_select_paths(entries: &[String]) -> Result<Vec<String>> {
    let selected = MultiSelect::new("Select paths to include:", entries.to_vec())
        .with_help_message("Type to filter, <space> to toggle, <enter> to confirm")
        .prompt()?;

    if selected.is_empty() {
        anyhow::bail!("No paths selected.");
    }

    Ok(selected)
}

/// Prompt the user for an output directory.
///
/// # Arguments
/// * `default` - Default value for the output directory
///
/// # Returns
/// The output directory path
pub fn prompt_output_dir(default: &str) -> Result<String> {
    let output = Text::new("Output directory:")
        .with_default(default)
        .prompt()?;

    Ok(output)
}

/// Run the interactive add workflow.
///
/// 1. Clone the repository in blobless mode
/// 2. List all entries (directories + files)
/// 3. Prompt user to select paths
/// 4. Prompt user for output directory
/// 5. Delegate to `add::run_add`
///
/// # Arguments
/// * `config_path` - Path to the config file
/// * `repo` - Git repository URL
/// * `rev` - Optional revision
/// * `name` - Optional dependency name
pub fn run_interactive_add(
    config_path: &Path,
    repo: &str,
    rev: Option<&str>,
    name: Option<&str>,
) -> Result<()> {
    let rev_str = rev.unwrap_or("HEAD");
    let entries = list_repo_entries(repo, rev_str)?;

    if entries.is_empty() {
        anyhow::bail!("No files found in the repository.");
    }

    let selected_paths = prompt_select_paths(&entries)?;

    let repo_name = add::extract_repo_name(repo)?;
    let default_out = format!("./vendor/{repo_name}");
    let out = prompt_output_dir(&default_out)?;

    add::run_add(config_path, repo, selected_paths, &out, name, rev)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collect_entries_basic() {
        let file_paths = vec![
            "proto/v1/user.proto".to_string(),
            "proto/v1/order.proto".to_string(),
            "README.md".to_string(),
        ];

        let result = collect_entries(&file_paths);

        let expected = vec![
            "README.md",
            "proto/",
            "proto/v1/",
            "proto/v1/order.proto",
            "proto/v1/user.proto",
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_collect_entries_empty() {
        let file_paths: Vec<String> = vec![];

        let result = collect_entries(&file_paths);

        assert!(result.is_empty());
    }

    #[test]
    fn test_collect_entries_root_files() {
        let file_paths = vec![
            "LICENSE".to_string(),
            "README.md".to_string(),
            "Cargo.toml".to_string(),
        ];

        let result = collect_entries(&file_paths);

        let expected = vec!["Cargo.toml", "LICENSE", "README.md"];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_collect_entries_nested() {
        let file_paths = vec![
            "a/b/c/d.txt".to_string(),
            "a/b/e.txt".to_string(),
            "a/f.txt".to_string(),
        ];

        let result = collect_entries(&file_paths);

        let expected = vec![
            "a/",
            "a/b/",
            "a/b/c/",
            "a/b/c/d.txt",
            "a/b/e.txt",
            "a/f.txt",
        ];
        assert_eq!(result, expected);
    }
}
