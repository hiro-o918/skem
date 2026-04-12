use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Strip the matching prefix from a file path based on dependency paths
///
/// # Arguments
/// * `file_path` - The absolute file path to process
/// * `dep_paths` - List of path prefixes to match against
///
/// # Returns
/// The relative path with the matching prefix stripped, or None if no prefix matches
///
/// # Example
/// ```
/// use std::path::PathBuf;
/// use skem::copy::strip_path_prefix;
///
/// let file_path = PathBuf::from("/tmp/repo/proto/v1/user.proto");
/// let dep_paths = vec!["proto/v1/".to_string()];
/// let result = strip_path_prefix(&file_path, &dep_paths);
/// assert_eq!(result, Some(PathBuf::from("user.proto")));
/// ```
pub fn strip_path_prefix(file_path: &Path, dep_paths: &[String]) -> Option<PathBuf> {
    // Try each dependency path as a potential prefix
    for dep_path in dep_paths {
        // Normalize the dep_path to ensure consistent matching
        let normalized_dep_path = if dep_path.ends_with('/') {
            dep_path.as_str()
        } else {
            // For paths without trailing slash, we'll handle them separately
            dep_path.as_str()
        };

        // Convert file_path to string for matching
        let file_path_str = file_path.to_str()?;

        // Search for the dep_path pattern within the file path
        if let Some(pos) = file_path_str.find(normalized_dep_path) {
            // Calculate where to start stripping (after the matched prefix)
            let strip_start = pos + normalized_dep_path.len();

            // Handle trailing slash: if the dep_path doesn't end with '/',
            // we need to skip one more character if it's a '/'
            let strip_start = if !normalized_dep_path.ends_with('/')
                && file_path_str.chars().nth(strip_start) == Some('/')
            {
                strip_start + 1
            } else {
                strip_start
            };

            // Extract the remaining path
            let stripped = &file_path_str[strip_start..];

            if !stripped.is_empty() {
                // Directory match: remaining subpath after the prefix
                return Some(PathBuf::from(stripped));
            } else {
                // File match: dep_path includes the filename itself, so return the filename
                let file_name = Path::new(normalized_dep_path).file_name()?;
                return Some(PathBuf::from(file_name));
            }
        }
    }

    // No matching prefix found
    None
}

/// Copy files from source directory to output directory with path stripping
///
/// # Arguments
/// * `source_dir` - Source directory containing files (e.g., temp checkout directory)
/// * `dep_paths` - List of path prefixes to match and strip
/// * `out_dir` - Output directory where files should be copied
///
/// # Returns
/// List of destination paths for successfully copied files
///
/// # Example
/// Copies files matching dep_paths from source_dir to out_dir,
/// stripping the matching prefix from the destination path.
pub fn copy_files(source_dir: &Path, dep_paths: &[String], out_dir: &Path) -> Result<Vec<PathBuf>> {
    let mut copied_files = Vec::new();

    // Recursively walk through all files in source directory
    for entry in WalkDir::new(source_dir).into_iter().filter_map(|e| e.ok()) {
        // Skip directories, only process files
        if !entry.file_type().is_file() {
            continue;
        }

        let file_path = entry.path();

        // Try to strip the prefix from the file path
        if let Some(stripped_path) = strip_path_prefix(file_path, dep_paths) {
            // Construct the destination path
            let dest_path = out_dir.join(&stripped_path);

            // Create parent directories if they don't exist
            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent).map_err(|e| {
                    anyhow::anyhow!("Failed to create parent directory {parent:?}: {e}")
                })?;
            }

            // Copy the file to the destination
            fs::copy(file_path, &dest_path).map_err(|e| {
                anyhow::anyhow!("Failed to copy file from {file_path:?} to {dest_path:?}: {e}")
            })?;

            copied_files.push(dest_path);
        }
    }

    Ok(copied_files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_strip_path_prefix_single_level_directory() {
        // Arrange: File in a single level directory matching the prefix
        let file_path = PathBuf::from("/tmp/repo/proto/user.proto");
        let dep_paths = vec!["proto/".to_string()];

        // Act: Strip the prefix
        let result = strip_path_prefix(&file_path, &dep_paths);

        // Assert: The "proto/" prefix should be stripped
        assert_eq!(result, Some(PathBuf::from("user.proto")));
    }

    #[test]
    fn test_strip_path_prefix_nested_directory() {
        // Arrange: File in a nested directory matching the prefix
        let file_path = PathBuf::from("/tmp/repo/proto/v1/user.proto");
        let dep_paths = vec!["proto/v1/".to_string()];

        // Act: Strip the prefix
        let result = strip_path_prefix(&file_path, &dep_paths);

        // Assert: The "proto/v1/" prefix should be stripped
        assert_eq!(result, Some(PathBuf::from("user.proto")));
    }

    #[test]
    fn test_strip_path_prefix_nested_file_structure() {
        // Arrange: File with nested structure after the prefix
        let file_path = PathBuf::from("/tmp/repo/proto/v1/services/auth/user.proto");
        let dep_paths = vec!["proto/v1/".to_string()];

        // Act: Strip the prefix
        let result = strip_path_prefix(&file_path, &dep_paths);

        // Assert: Only "proto/v1/" should be stripped, keeping "services/auth/"
        assert_eq!(result, Some(PathBuf::from("services/auth/user.proto")));
    }

    #[test]
    fn test_strip_path_prefix_multiple_paths_first_match() {
        // Arrange: Multiple possible prefixes, file matches the first one
        let file_path = PathBuf::from("/tmp/repo/proto/v1/user.proto");
        let dep_paths = vec!["proto/v1/".to_string(), "proto/v2/".to_string()];

        // Act: Strip the prefix
        let result = strip_path_prefix(&file_path, &dep_paths);

        // Assert: The first matching prefix "proto/v1/" should be stripped
        assert_eq!(result, Some(PathBuf::from("user.proto")));
    }

    #[test]
    fn test_strip_path_prefix_multiple_paths_second_match() {
        // Arrange: Multiple possible prefixes, file matches the second one
        let file_path = PathBuf::from("/tmp/repo/proto/v2/user.proto");
        let dep_paths = vec!["proto/v1/".to_string(), "proto/v2/".to_string()];

        // Act: Strip the prefix
        let result = strip_path_prefix(&file_path, &dep_paths);

        // Assert: The second matching prefix "proto/v2/" should be stripped
        assert_eq!(result, Some(PathBuf::from("user.proto")));
    }

    #[test]
    fn test_strip_path_prefix_no_match() {
        // Arrange: File path that doesn't match any prefix
        let file_path = PathBuf::from("/tmp/repo/src/main.rs");
        let dep_paths = vec!["proto/".to_string()];

        // Act: Strip the prefix
        let result = strip_path_prefix(&file_path, &dep_paths);

        // Assert: Should return None when no prefix matches
        assert_eq!(result, None);
    }

    #[test]
    fn test_strip_path_prefix_empty_paths() {
        // Arrange: Empty dependency paths list
        let file_path = PathBuf::from("/tmp/repo/proto/user.proto");
        let dep_paths: Vec<String> = vec![];

        // Act: Strip the prefix
        let result = strip_path_prefix(&file_path, &dep_paths);

        // Assert: Should return None when paths list is empty
        assert_eq!(result, None);
    }

    #[test]
    fn test_strip_path_prefix_without_trailing_slash() {
        // Arrange: dep_path without trailing slash
        let file_path = PathBuf::from("/tmp/repo/proto/user.proto");
        let dep_paths = vec!["proto".to_string()];

        // Act: Strip the prefix
        let result = strip_path_prefix(&file_path, &dep_paths);

        // Assert: Should handle paths without trailing slash
        assert_eq!(result, Some(PathBuf::from("user.proto")));
    }

    #[test]
    fn test_strip_path_prefix_exact_file_path() {
        // Arrange: dep_path points directly to a file (not a directory)
        let file_path = PathBuf::from("/tmp/repo/backend/api/openapi3.yaml");
        let dep_paths = vec!["backend/api/openapi3.yaml".to_string()];

        // Act: Strip the prefix
        let result = strip_path_prefix(&file_path, &dep_paths);

        // Assert: Should return just the filename when dep_path is an exact file path
        assert_eq!(result, Some(PathBuf::from("openapi3.yaml")));
    }

    #[test]
    fn test_strip_path_prefix_exact_file_path_nested() {
        // Arrange: dep_path points directly to a deeply nested file
        let file_path = PathBuf::from("/tmp/repo/backend/apiapp/gen/http/openapi3.yaml");
        let dep_paths = vec!["backend/apiapp/gen/http/openapi3.yaml".to_string()];

        // Act: Strip the prefix
        let result = strip_path_prefix(&file_path, &dep_paths);

        // Assert: Should return just the filename
        assert_eq!(result, Some(PathBuf::from("openapi3.yaml")));
    }

    #[test]
    fn test_copy_files_single_file() {
        // Arrange: Create source directory with a single file
        let source_dir = TempDir::new().expect("Should create temp dir");
        let proto_dir = source_dir.path().join("proto");
        fs::create_dir_all(&proto_dir).expect("Should create proto dir");
        fs::write(proto_dir.join("user.proto"), "message User {}").expect("Should write file");

        let out_dir = TempDir::new().expect("Should create output dir");
        let dep_paths = vec!["proto/".to_string()];

        // Act: Copy files
        let result = copy_files(source_dir.path(), &dep_paths, out_dir.path());

        // Assert: File should be copied with stripped path, returns copied file paths
        assert!(result.is_ok(), "copy_files should succeed");
        let copied_files = result.unwrap();
        let expected = vec![out_dir.path().join("user.proto")];
        assert_eq!(copied_files, expected);
        assert!(
            out_dir.path().join("user.proto").exists(),
            "user.proto should be copied to output directory"
        );

        let content =
            fs::read_to_string(out_dir.path().join("user.proto")).expect("Should read copied file");
        assert_eq!(content, "message User {}");
    }

    #[test]
    fn test_copy_files_nested_structure() {
        // Arrange: Create source directory with nested file structure
        let source_dir = TempDir::new().expect("Should create temp dir");
        let nested_dir = source_dir.path().join("proto/v1/services/auth");
        fs::create_dir_all(&nested_dir).expect("Should create nested dirs");
        fs::write(nested_dir.join("user.proto"), "message User {}").expect("Should write file");

        let out_dir = TempDir::new().expect("Should create output dir");
        let dep_paths = vec!["proto/v1/".to_string()];

        // Act: Copy files
        let result = copy_files(source_dir.path(), &dep_paths, out_dir.path());

        // Assert: File should be copied preserving structure after stripped prefix
        assert!(result.is_ok(), "copy_files should succeed");
        let copied_files = result.unwrap();
        let expected = vec![out_dir.path().join("services/auth/user.proto")];
        assert_eq!(copied_files, expected);
    }

    #[test]
    fn test_copy_files_multiple_files() {
        // Arrange: Create source directory with multiple files
        let source_dir = TempDir::new().expect("Should create temp dir");
        let proto_dir = source_dir.path().join("proto");
        fs::create_dir_all(&proto_dir).expect("Should create proto dir");
        fs::write(proto_dir.join("user.proto"), "message User {}").expect("Should write file");
        fs::write(proto_dir.join("post.proto"), "message Post {}").expect("Should write file");

        let out_dir = TempDir::new().expect("Should create output dir");
        let dep_paths = vec!["proto/".to_string()];

        // Act: Copy files
        let result = copy_files(source_dir.path(), &dep_paths, out_dir.path());

        // Assert: All files should be copied
        assert!(result.is_ok(), "copy_files should succeed");
        let mut copied_files = result.unwrap();
        copied_files.sort();
        let mut expected = vec![
            out_dir.path().join("post.proto"),
            out_dir.path().join("user.proto"),
        ];
        expected.sort();
        assert_eq!(copied_files, expected);
    }

    #[test]
    fn test_copy_files_creates_parent_directories() {
        // Arrange: Create source directory with deeply nested file
        let source_dir = TempDir::new().expect("Should create temp dir");
        let nested_dir = source_dir.path().join("proto/v1/a/b/c");
        fs::create_dir_all(&nested_dir).expect("Should create nested dirs");
        fs::write(nested_dir.join("deep.proto"), "message Deep {}").expect("Should write file");

        let out_dir = TempDir::new().expect("Should create output dir");
        let dep_paths = vec!["proto/v1/".to_string()];

        // Act: Copy files
        let result = copy_files(source_dir.path(), &dep_paths, out_dir.path());

        // Assert: Parent directories should be created automatically
        assert!(result.is_ok(), "copy_files should succeed");
        let copied_files = result.unwrap();
        let expected = vec![out_dir.path().join("a/b/c/deep.proto")];
        assert_eq!(copied_files, expected);
    }

    #[test]
    fn test_copy_files_no_matching_files() {
        // Arrange: Create source directory with files that don't match dep_paths
        let source_dir = TempDir::new().expect("Should create temp dir");
        let src_dir = source_dir.path().join("src");
        fs::create_dir_all(&src_dir).expect("Should create src dir");
        fs::write(src_dir.join("main.rs"), "fn main() {}").expect("Should write file");

        let out_dir = TempDir::new().expect("Should create output dir");
        let dep_paths = vec!["proto/".to_string()]; // Looking for proto, but only src exists

        // Act: Copy files
        let result = copy_files(source_dir.path(), &dep_paths, out_dir.path());

        // Assert: No files should be copied
        assert!(result.is_ok(), "copy_files should succeed");
        let copied_files = result.unwrap();
        assert!(
            copied_files.is_empty(),
            "Should copy 0 files when no matches"
        );
    }

    #[test]
    fn test_copy_files_exact_file_path() {
        // Arrange: Create source directory with a file specified by exact path in dep_paths
        let source_dir = TempDir::new().expect("Should create temp dir");
        let nested_dir = source_dir.path().join("backend/api");
        fs::create_dir_all(&nested_dir).expect("Should create nested dirs");
        fs::write(nested_dir.join("openapi3.yaml"), "openapi: 3.0.0").expect("Should write file");

        let out_dir = TempDir::new().expect("Should create output dir");
        let dep_paths = vec!["backend/api/openapi3.yaml".to_string()];

        // Act: Copy files
        let result = copy_files(source_dir.path(), &dep_paths, out_dir.path());

        // Assert: File should be copied with just the filename (no path prefix)
        assert!(result.is_ok(), "copy_files should succeed");
        let copied_files = result.unwrap();
        let expected = vec![out_dir.path().join("openapi3.yaml")];
        assert_eq!(copied_files, expected);
        assert!(
            out_dir.path().join("openapi3.yaml").exists(),
            "openapi3.yaml should be copied to output directory"
        );
        let content = fs::read_to_string(out_dir.path().join("openapi3.yaml"))
            .expect("Should read copied file");
        assert_eq!(content, "openapi: 3.0.0");
    }

    #[test]
    fn test_copy_files_multiple_dep_paths() {
        // Arrange: Create source directory with files in different paths
        let source_dir = TempDir::new().expect("Should create temp dir");

        let proto_v1_dir = source_dir.path().join("proto/v1");
        fs::create_dir_all(&proto_v1_dir).expect("Should create proto/v1 dir");
        fs::write(proto_v1_dir.join("user.proto"), "message User {}").expect("Should write file");

        let proto_v2_dir = source_dir.path().join("proto/v2");
        fs::create_dir_all(&proto_v2_dir).expect("Should create proto/v2 dir");
        fs::write(proto_v2_dir.join("post.proto"), "message Post {}").expect("Should write file");

        let out_dir = TempDir::new().expect("Should create output dir");
        let dep_paths = vec!["proto/v1/".to_string(), "proto/v2/".to_string()];

        // Act: Copy files
        let result = copy_files(source_dir.path(), &dep_paths, out_dir.path());

        // Assert: Files from both paths should be copied
        assert!(result.is_ok(), "copy_files should succeed");
        let mut copied_files = result.unwrap();
        copied_files.sort();
        let mut expected = vec![
            out_dir.path().join("post.proto"),
            out_dir.path().join("user.proto"),
        ];
        expected.sort();
        assert_eq!(copied_files, expected);
    }
}
