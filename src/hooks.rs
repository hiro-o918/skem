use anyhow::Result;
use std::process::Command;

/// Execute hooks in sequential order
///
/// # Arguments
/// * `hooks` - List of shell commands to execute
///
/// # Returns
/// Result that succeeds if all hooks execute successfully, or fails if any hook fails
///
/// # Example
/// ```
/// use skem::hooks::execute_hooks;
///
/// let hooks = vec!["echo 'done'".to_string()];
/// let result = execute_hooks(&hooks);
/// assert!(result.is_ok());
/// ```
pub fn execute_hooks(hooks: &[String]) -> Result<()> {
    execute_hooks_with_env(hooks, &[])
}

/// Execute hooks in sequential order with additional environment variables
///
/// # Arguments
/// * `hooks` - List of shell commands to execute
/// * `env_vars` - Environment variables to pass to hook processes
///
/// # Returns
/// Result that succeeds if all hooks execute successfully, or fails if any hook fails
pub fn execute_hooks_with_env(hooks: &[String], env_vars: &[(&str, &str)]) -> Result<()> {
    for hook in hooks {
        execute_hook(hook, env_vars)?;
    }
    Ok(())
}

/// Execute a single hook command
///
/// # Arguments
/// * `hook` - Shell command to execute
/// * `env_vars` - Environment variables to pass to the hook process
///
/// # Returns
/// Result that succeeds if the command succeeds, or fails if the command fails
fn execute_hook(hook: &str, env_vars: &[(&str, &str)]) -> Result<()> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(hook)
        .envs(env_vars.iter().copied())
        .output()
        .map_err(|e| anyhow::anyhow!("Failed to execute hook '{hook}': {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(anyhow::anyhow!(
            "Hook '{}' failed with exit code {:?}\nStdout: {}\nStderr: {}",
            hook,
            output.status.code(),
            stdout,
            stderr
        ));
    }

    // Print hook output to stdout for visibility
    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.is_empty() {
        println!("{stdout}");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_hooks_empty_list() {
        // Arrange: Empty hooks list
        let hooks: Vec<String> = vec![];

        // Act: Execute hooks
        let result = execute_hooks(&hooks);

        // Assert: Should succeed with no hooks
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_hooks_single_hook() {
        // Arrange: Single hook that succeeds
        let hooks = vec!["echo 'test'".to_string()];

        // Act: Execute hooks
        let result = execute_hooks(&hooks);

        // Assert: Should succeed
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_hooks_multiple_hooks() {
        // Arrange: Multiple hooks that succeed
        let hooks = vec![
            "echo 'first'".to_string(),
            "echo 'second'".to_string(),
            "echo 'third'".to_string(),
        ];

        // Act: Execute hooks
        let result = execute_hooks(&hooks);

        // Assert: All hooks should execute successfully
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_hooks_fails_on_first_failure() {
        // Arrange: Hooks where the second one fails
        let hooks = vec![
            "echo 'first'".to_string(),
            "exit 1".to_string(),       // This hook fails
            "echo 'third'".to_string(), // This should not execute
        ];

        // Act: Execute hooks
        let result = execute_hooks(&hooks);

        // Assert: Should fail and not continue to next hook
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("failed with exit code"));
    }

    #[test]
    fn test_execute_hooks_command_not_found() {
        // Arrange: Hook that references non-existent command
        let hooks = vec!["nonexistent_command_12345".to_string()];

        // Act: Execute hooks
        let result = execute_hooks(&hooks);

        // Assert: Should fail
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_hook_with_output() {
        // Arrange: Hook that produces output
        let hooks = vec!["echo 'hook output'".to_string()];

        // Act: Execute hooks (output goes to stdout)
        let result = execute_hooks(&hooks);

        // Assert: Should succeed
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_hook_with_complex_command() {
        // Arrange: Complex hook with pipes and redirections
        let hooks = vec!["echo 'test' | grep 'test'".to_string()];

        // Act: Execute hooks
        let result = execute_hooks(&hooks);

        // Assert: Should succeed
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_hook_with_exit_code() {
        // Arrange: Hook with specific exit code
        let hooks = vec!["exit 42".to_string()];

        // Act: Execute hooks
        let result = execute_hooks(&hooks);

        // Assert: Should fail with the specific exit code
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("exit code"));
    }

    #[test]
    fn test_execute_hooks_with_env_vars() {
        // Arrange: Hook that reads environment variable and writes to a temp file
        let temp_dir = tempfile::TempDir::new().expect("Should create temp dir");
        let output_path = temp_dir.path().join("env_output.txt");
        let output_path_str = output_path.to_str().unwrap();
        let hooks = vec![format!("echo $SKEM_SYNCED_FILES > {output_path_str}")];
        let env_vars = vec![(
            "SKEM_SYNCED_FILES",
            "vendor/proto/user.proto vendor/proto/post.proto",
        )];

        // Act: Execute hooks with environment variables
        let result = execute_hooks_with_env(&hooks, &env_vars);

        // Assert: Hook should have access to the environment variable
        assert!(result.is_ok());
        let content = std::fs::read_to_string(&output_path).expect("Should read output file");
        assert_eq!(
            content.trim(),
            "vendor/proto/user.proto vendor/proto/post.proto"
        );
    }

    #[test]
    fn test_execute_hooks_with_env_vars_empty_env() {
        // Arrange: Hook with no environment variables
        let hooks = vec!["echo 'test'".to_string()];
        let env_vars: Vec<(&str, &str)> = vec![];

        // Act: Execute hooks with empty env vars
        let result = execute_hooks_with_env(&hooks, &env_vars);

        // Assert: Should succeed
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_hooks_with_env_vars_multiple_vars() {
        // Arrange: Hook that reads multiple environment variables
        let temp_dir = tempfile::TempDir::new().expect("Should create temp dir");
        let output_path = temp_dir.path().join("env_output.txt");
        let output_path_str = output_path.to_str().unwrap();
        let hooks = vec![format!(
            "echo \"$SKEM_SYNCED_FILES|$SKEM_OTHER\" > {output_path_str}"
        )];
        let env_vars = vec![
            ("SKEM_SYNCED_FILES", "file1.proto file2.proto"),
            ("SKEM_OTHER", "extra_value"),
        ];

        // Act: Execute hooks with multiple environment variables
        let result = execute_hooks_with_env(&hooks, &env_vars);

        // Assert: Hook should have access to all environment variables
        assert!(result.is_ok());
        let content = std::fs::read_to_string(&output_path).expect("Should read output file");
        assert_eq!(content.trim(), "file1.proto file2.proto|extra_value");
    }
}
