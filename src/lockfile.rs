use crate::config::{LockEntry, Lockfile};
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

/// Create a new lockfile with the specified dependency removed
///
/// # Arguments
/// * `lockfile` - Source lockfile
/// * `name` - Dependency name to remove
///
/// # Returns
/// New Lockfile with the specified entry removed
pub fn remove_lockfile_entry(lockfile: &Lockfile, name: &str) -> Lockfile {
    Lockfile {
        locks: lockfile
            .locks
            .iter()
            .filter(|entry| entry.name != name)
            .cloned()
            .collect(),
    }
}

/// Create a new lockfile with a single dependency updated
///
/// # Arguments
/// * `lockfile` - Source lockfile
/// * `name` - Dependency name
/// * `sha` - New SHA to set
///
/// # Returns
/// New Lockfile with the specified entry updated or added
pub fn update_lockfile_entry(lockfile: &Lockfile, name: &str, sha: &str) -> Lockfile {
    let mut found = false;
    let mut locks: Vec<LockEntry> = lockfile
        .locks
        .iter()
        .map(|entry| {
            if entry.name == name {
                found = true;
                LockEntry {
                    name: name.to_string(),
                    sha: sha.to_string(),
                }
            } else {
                entry.clone()
            }
        })
        .collect();

    if !found {
        locks.push(LockEntry {
            name: name.to_string(),
            sha: sha.to_string(),
        });
    }

    Lockfile { locks }
}

/// Create a new lockfile with multiple dependencies updated
///
/// # Arguments
/// * `lockfile` - Source lockfile
/// * `updates` - Iterator of (name, sha) tuples to update
///
/// # Returns
/// New Lockfile with all specified entries updated or added
pub fn update_lockfile_entries<'a, I>(lockfile: &Lockfile, updates: I) -> Lockfile
where
    I: IntoIterator<Item = (&'a str, &'a str)>,
{
    updates
        .into_iter()
        .fold(lockfile.clone(), |acc, (name, sha)| {
            update_lockfile_entry(&acc, name, sha)
        })
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
        let lockfile = Lockfile { locks: vec![] };

        // Act: Update lockfile with new entry
        let result = update_lockfile_entry(&lockfile, "new-dep", "sha123");

        // Assert: New entry should be added
        let expected = Lockfile {
            locks: vec![LockEntry {
                name: "new-dep".to_string(),
                sha: "sha123".to_string(),
            }],
        };
        assert_eq!(result, expected);
    }

    #[test]
    fn test_update_lockfile_entry_updates_existing_entry() {
        // Arrange: Lockfile with existing entry
        let lockfile = Lockfile {
            locks: vec![LockEntry {
                name: "existing-dep".to_string(),
                sha: "old-sha".to_string(),
            }],
        };

        // Act: Update existing entry
        let result = update_lockfile_entry(&lockfile, "existing-dep", "new-sha");

        // Assert: Entry should be updated, length unchanged
        let expected = Lockfile {
            locks: vec![LockEntry {
                name: "existing-dep".to_string(),
                sha: "new-sha".to_string(),
            }],
        };
        assert_eq!(result, expected);
    }

    #[test]
    fn test_update_lockfile_entry_does_not_modify_original() {
        // Arrange: Lockfile with existing entry
        let lockfile = Lockfile {
            locks: vec![LockEntry {
                name: "dep".to_string(),
                sha: "old-sha".to_string(),
            }],
        };

        // Act: Update entry
        let _ = update_lockfile_entry(&lockfile, "dep", "new-sha");

        // Assert: Original lockfile should be unchanged
        let expected = Lockfile {
            locks: vec![LockEntry {
                name: "dep".to_string(),
                sha: "old-sha".to_string(),
            }],
        };
        assert_eq!(lockfile, expected);
    }

    #[test]
    fn test_update_lockfile_entry_multiple_entries() {
        // Arrange: Lockfile with multiple entries
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

        // Act: Update one entry
        let result = update_lockfile_entry(&lockfile, "dep1", "new-sha1");

        // Assert: Only first entry should be updated
        let expected = Lockfile {
            locks: vec![
                LockEntry {
                    name: "dep1".to_string(),
                    sha: "new-sha1".to_string(),
                },
                LockEntry {
                    name: "dep2".to_string(),
                    sha: "sha2".to_string(),
                },
            ],
        };
        assert_eq!(result, expected);
    }

    #[test]
    fn test_update_lockfile_entries_batch_update() {
        // Arrange: Empty lockfile
        let lockfile = Lockfile { locks: vec![] };

        // Act: Update multiple entries at once
        let updates = vec![("dep1", "sha1"), ("dep2", "sha2"), ("dep3", "sha3")];
        let result = update_lockfile_entries(&lockfile, updates);

        // Assert: All entries should be added
        let expected = Lockfile {
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
        assert_eq!(result, expected);
    }

    #[test]
    fn test_update_lockfile_entries_mixed_add_and_update() {
        // Arrange: Lockfile with one entry
        let lockfile = Lockfile {
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
        let result = update_lockfile_entries(&lockfile, updates);

        // Assert: Should have updated existing and added new entries
        let expected = Lockfile {
            locks: vec![
                LockEntry {
                    name: "existing".to_string(),
                    sha: "new-sha".to_string(),
                },
                LockEntry {
                    name: "new-dep1".to_string(),
                    sha: "sha1".to_string(),
                },
                LockEntry {
                    name: "new-dep2".to_string(),
                    sha: "sha2".to_string(),
                },
            ],
        };
        assert_eq!(result, expected);
    }

    #[test]
    fn test_remove_lockfile_entry_existing() {
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

        let result = remove_lockfile_entry(&lockfile, "dep1");
        let expected = Lockfile {
            locks: vec![LockEntry {
                name: "dep2".to_string(),
                sha: "sha2".to_string(),
            }],
        };
        assert_eq!(result, expected);
    }

    #[test]
    fn test_remove_lockfile_entry_nonexistent() {
        let lockfile = Lockfile {
            locks: vec![LockEntry {
                name: "dep1".to_string(),
                sha: "sha1".to_string(),
            }],
        };

        let result = remove_lockfile_entry(&lockfile, "nonexistent");
        // Should remain unchanged
        assert_eq!(result, lockfile);
    }

    #[test]
    fn test_remove_lockfile_entry_does_not_modify_original() {
        let lockfile = Lockfile {
            locks: vec![LockEntry {
                name: "dep1".to_string(),
                sha: "sha1".to_string(),
            }],
        };

        let _ = remove_lockfile_entry(&lockfile, "dep1");
        // Original should be unchanged
        let expected = Lockfile {
            locks: vec![LockEntry {
                name: "dep1".to_string(),
                sha: "sha1".to_string(),
            }],
        };
        assert_eq!(lockfile, expected);
    }
}
