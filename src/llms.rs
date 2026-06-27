use anyhow::Result;

/// Static, English, LLM-oriented usage guide for skem.
///
/// The text is intentionally self-contained so that an LLM agent can
/// understand how to use skem from a single `skem llms` invocation,
/// without needing to fetch the README or the docs directory.
pub const GUIDE: &str = include_str!("llms_guide.md");

/// Return the guide content. Kept as a function so future variants
/// (filtered/short forms) can be added without changing call sites.
pub fn guide() -> &'static str {
    GUIDE
}

/// Print the guide to stdout.
pub fn run() -> Result<()> {
    println!("{}", guide());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_guide_is_not_empty() {
        let text = guide();
        assert!(!text.is_empty());
    }

    #[test]
    fn test_guide_documents_all_subcommands() {
        let text = guide();
        for cmd in [
            "skem init",
            "skem schema",
            "skem sync",
            "skem add",
            "skem rm",
            "skem ls",
            "skem check",
            "skem self-update",
            "skem llms",
        ] {
            assert!(
                text.contains(cmd),
                "guide should mention `{cmd}` but did not"
            );
        }
    }

    #[test]
    fn test_guide_documents_config_file_format() {
        let text = guide();
        assert!(text.contains(".skem.yaml"));
        assert!(text.contains(".skem.lock"));
        for field in [
            "deps:",
            "name:",
            "repo:",
            "paths:",
            "out:",
            "rev:",
            "hooks:",
            "post_hooks:",
        ] {
            assert!(
                text.contains(field),
                "guide should document config field `{field}` but did not"
            );
        }
    }

    #[test]
    fn test_guide_documents_hook_environment_variable() {
        let text = guide();
        assert!(
            text.contains("SKEM_SYNCED_FILES"),
            "guide should document the SKEM_SYNCED_FILES env var available to hooks"
        );
    }

    #[test]
    fn test_guide_documents_sync_flags() {
        let text = guide();
        assert!(text.contains("--force"));
        assert!(text.contains("--hooks-only"));
    }

    #[test]
    fn test_guide_documents_add_interactive_mode() {
        let text = guide();
        assert!(text.to_lowercase().contains("interactive"));
        assert!(text.contains("--paths"));
        assert!(text.contains("--out"));
        assert!(text.contains("--repo"));
    }

    #[test]
    fn test_guide_mentions_exit_code_semantics_for_check() {
        let text = guide();
        assert!(text.contains("check"));
        assert!(
            text.contains("exit") || text.contains("Exit"),
            "guide should describe `skem check` exit code behavior"
        );
    }

    #[test]
    fn test_run_writes_guide_without_error() {
        // run() prints to stdout; just verify it returns Ok and the
        // underlying guide content is non-empty.
        let result = run();
        assert!(result.is_ok());
        assert!(!guide().is_empty());
    }
}
