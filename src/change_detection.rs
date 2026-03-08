use crate::config::Lockfile;
use anyhow::Result;
use std::fs;
use std::path::Path;

/// Read lockfile from the specified path
///
/// # Arguments
/// * `path` - Path to the lockfile
///
/// # Returns
/// Lockfile if it exists, otherwise an empty Lockfile
pub fn read_lockfile(path: &Path) -> Result<Lockfile> {
    if !path.exists() {
        return Ok(Lockfile { locks: vec![] });
    }

    let content = fs::read_to_string(path)?;
    let lockfile: Lockfile = serde_yaml::from_str(&content)?;
    Ok(lockfile)
}

/// Write lockfile to the specified path
///
/// # Arguments
/// * `path` - Path to the lockfile
/// * `lockfile` - Lockfile to write
pub fn write_lockfile(path: &Path, lockfile: &Lockfile) -> Result<()> {
    let content = serde_yaml::to_string(lockfile)?;
    fs::write(path, content)?;
    Ok(())
}

/// Check if a dependency has changed by comparing SHA
///
/// # Arguments
/// * `name` - Dependency name
/// * `current_sha` - Current SHA from git ls-remote
/// * `lockfile` - Lockfile containing previous SHA
///
/// # Returns
/// true if the dependency has changed (or is new), false otherwise
pub fn has_changed(name: &str, current_sha: &str, lockfile: &Lockfile) -> bool {
    lockfile
        .locks
        .iter()
        .find(|entry| entry.name == name)
        .is_none_or(|entry| entry.sha != current_sha)
}

/// Update a single dependency in the lockfile
///
/// # Arguments
/// * `lockfile` - Lockfile to update (mutable)
/// * `name` - Dependency name
/// * `sha` - New SHA to set
pub fn update_lockfile_entry(lockfile: &mut Lockfile, name: &str, sha: &str) {
    use crate::config::LockEntry;

    // Try to find and update existing entry
    if let Some(entry) = lockfile.locks.iter_mut().find(|e| e.name == name) {
        entry.sha = sha.to_string();
    } else {
        // Add new entry if not found
        lockfile.locks.push(LockEntry {
            name: name.to_string(),
            sha: sha.to_string(),
        });
    }
}

/// Update multiple dependencies in the lockfile
///
/// # Arguments
/// * `lockfile` - Lockfile to update (mutable)
/// * `updates` - Iterator of (name, sha) tuples to update
pub fn update_lockfile_entries<'a, I>(lockfile: &mut Lockfile, updates: I)
where
    I: IntoIterator<Item = (&'a str, &'a str)>,
{
    for (name, sha) in updates {
        update_lockfile_entry(lockfile, name, sha);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::LockEntry;
    use tempfile::TempDir;

    #[test]
    fn test_read_lockfile_when_file_exists() {
        // Arrange: Create a temporary lockfile
        let temp_dir = TempDir::new().unwrap();
        let lockfile_path = temp_dir.path().join(".skem.lock");

        let yaml = r#"
locks:
  - name: example-api
    sha: "abc123def456"
  - name: another-dep
    sha: "789ghi012jkl"
"#;
        fs::write(&lockfile_path, yaml).unwrap();

        // Act: Read the lockfile
        let lockfile = read_lockfile(&lockfile_path).unwrap();

        // Assert: Compare complete object
        let expected = Lockfile {
            locks: vec![
                LockEntry {
                    name: "example-api".to_string(),
                    sha: "abc123def456".to_string(),
                },
                LockEntry {
                    name: "another-dep".to_string(),
                    sha: "789ghi012jkl".to_string(),
                },
            ],
        };
        assert_eq!(lockfile, expected);
    }

    #[test]
    fn test_read_lockfile_when_file_does_not_exist() {
        // Arrange: Use a path that doesn't exist
        let temp_dir = TempDir::new().unwrap();
        let lockfile_path = temp_dir.path().join(".skem.lock");

        // Act: Read the lockfile
        let lockfile = read_lockfile(&lockfile_path).unwrap();

        // Assert: Should return empty Lockfile
        let expected = Lockfile { locks: vec![] };
        assert_eq!(lockfile, expected);
    }

    #[test]
    fn test_write_lockfile_creates_file() {
        // Arrange: Prepare lockfile data
        let temp_dir = TempDir::new().unwrap();
        let lockfile_path = temp_dir.path().join(".skem.lock");

        let lockfile = Lockfile {
            locks: vec![
                LockEntry {
                    name: "dep1".to_string(),
                    sha: "sha1".to_string(),
                },
                LockEntry {
                    name: "dep2".to_string(),
                    sha: "sha2".to_string(),
                },
            ],
        };

        // Act: Write lockfile
        write_lockfile(&lockfile_path, &lockfile).unwrap();

        // Assert: File should exist and be readable
        assert!(lockfile_path.exists());

        let read_lockfile = read_lockfile(&lockfile_path).unwrap();
        assert_eq!(read_lockfile, lockfile);
    }

    #[test]
    fn test_write_lockfile_overwrites_existing() {
        // Arrange: Create initial lockfile
        let temp_dir = TempDir::new().unwrap();
        let lockfile_path = temp_dir.path().join(".skem.lock");

        let initial_lockfile = Lockfile {
            locks: vec![LockEntry {
                name: "old-dep".to_string(),
                sha: "old-sha".to_string(),
            }],
        };
        write_lockfile(&lockfile_path, &initial_lockfile).unwrap();

        // Act: Overwrite with new lockfile
        let new_lockfile = Lockfile {
            locks: vec![LockEntry {
                name: "new-dep".to_string(),
                sha: "new-sha".to_string(),
            }],
        };
        write_lockfile(&lockfile_path, &new_lockfile).unwrap();

        // Assert: Should contain new data
        let read_lockfile = read_lockfile(&lockfile_path).unwrap();
        assert_eq!(read_lockfile, new_lockfile);
    }

    #[test]
    fn test_has_changed_returns_true_for_new_dependency() {
        // Arrange: Empty lockfile
        let lockfile = Lockfile { locks: vec![] };

        // Act: Check if dependency has changed
        let changed = has_changed("new-dep", "abc123", &lockfile);

        // Assert: Should return true (new dependency)
        assert!(changed);
    }

    #[test]
    fn test_has_changed_returns_true_when_sha_differs() {
        // Arrange: Lockfile with old SHA
        let lockfile = Lockfile {
            locks: vec![LockEntry {
                name: "existing-dep".to_string(),
                sha: "old-sha".to_string(),
            }],
        };

        // Act: Check with different SHA
        let changed = has_changed("existing-dep", "new-sha", &lockfile);

        // Assert: Should return true (SHA changed)
        assert!(changed);
    }

    #[test]
    fn test_has_changed_returns_false_when_sha_matches() {
        // Arrange: Lockfile with current SHA
        let lockfile = Lockfile {
            locks: vec![LockEntry {
                name: "existing-dep".to_string(),
                sha: "current-sha".to_string(),
            }],
        };

        // Act: Check with same SHA
        let changed = has_changed("existing-dep", "current-sha", &lockfile);

        // Assert: Should return false (no change)
        assert!(!changed);
    }

    #[test]
    fn test_has_changed_with_multiple_dependencies() {
        // Arrange: Lockfile with multiple dependencies
        let lockfile = Lockfile {
            locks: vec![
                LockEntry {
                    name: "dep1".to_string(),
                    sha: "sha1".to_string(),
                },
                LockEntry {
                    name: "dep2".to_string(),
                    sha: "sha2".to_string(),
                },
                LockEntry {
                    name: "dep3".to_string(),
                    sha: "sha3".to_string(),
                },
            ],
        };

        // Assert: Each dependency behaves correctly
        assert!(!has_changed("dep1", "sha1", &lockfile)); // No change
        assert!(has_changed("dep2", "new-sha", &lockfile)); // Changed
        assert!(has_changed("dep4", "any-sha", &lockfile)); // New dependency
    }

    #[test]
    fn test_update_lockfile_entry_adds_new_entry() {
        // Arrange: Empty lockfile
        let mut lockfile = Lockfile { locks: vec![] };

        // Act: Update lockfile with new entry
        update_lockfile_entry(&mut lockfile, "new-dep", "sha123");

        // Assert: New entry should be added
        assert_eq!(lockfile.locks.len(), 1);
        assert_eq!(lockfile.locks[0].name, "new-dep");
        assert_eq!(lockfile.locks[0].sha, "sha123");
    }

    #[test]
    fn test_update_lockfile_entry_updates_existing_entry() {
        // Arrange: Lockfile with existing entry
        let mut lockfile = Lockfile {
            locks: vec![LockEntry {
                name: "existing-dep".to_string(),
                sha: "old-sha".to_string(),
            }],
        };

        // Act: Update existing entry
        update_lockfile_entry(&mut lockfile, "existing-dep", "new-sha");

        // Assert: Entry should be updated, length unchanged
        assert_eq!(lockfile.locks.len(), 1);
        assert_eq!(lockfile.locks[0].sha, "new-sha");
    }

    #[test]
    fn test_update_lockfile_entry_multiple_entries() {
        // Arrange: Lockfile with multiple entries
        let mut lockfile = Lockfile {
            locks: vec![
                LockEntry {
                    name: "dep1".to_string(),
                    sha: "sha1".to_string(),
                },
                LockEntry {
                    name: "dep2".to_string(),
                    sha: "sha2".to_string(),
                },
            ],
        };

        // Act: Update one entry
        update_lockfile_entry(&mut lockfile, "dep1", "new-sha1");

        // Assert: Only first entry should be updated
        assert_eq!(lockfile.locks.len(), 2);
        assert_eq!(lockfile.locks[0].sha, "new-sha1");
        assert_eq!(lockfile.locks[1].sha, "sha2");
    }

    #[test]
    fn test_update_lockfile_entries_batch_update() {
        // Arrange: Empty lockfile
        let mut lockfile = Lockfile { locks: vec![] };

        // Act: Update multiple entries at once
        let updates = vec![("dep1", "sha1"), ("dep2", "sha2"), ("dep3", "sha3")];
        update_lockfile_entries(&mut lockfile, updates);

        // Assert: All entries should be added
        assert_eq!(lockfile.locks.len(), 3);
        assert_eq!(lockfile.locks[0].name, "dep1");
        assert_eq!(lockfile.locks[1].name, "dep2");
        assert_eq!(lockfile.locks[2].name, "dep3");
    }

    #[test]
    fn test_update_lockfile_entries_mixed_add_and_update() {
        // Arrange: Lockfile with one entry
        let mut lockfile = Lockfile {
            locks: vec![LockEntry {
                name: "existing".to_string(),
                sha: "old-sha".to_string(),
            }],
        };

        // Act: Update existing and add new entries
        let updates = vec![
            ("existing", "new-sha"),
            ("new-dep1", "sha1"),
            ("new-dep2", "sha2"),
        ];
        update_lockfile_entries(&mut lockfile, updates);

        // Assert: Should have updated existing and added new entries
        assert_eq!(lockfile.locks.len(), 3);
        assert_eq!(lockfile.locks[0].sha, "new-sha");
        assert_eq!(lockfile.locks[1].name, "new-dep1");
        assert_eq!(lockfile.locks[2].name, "new-dep2");
    }
}
