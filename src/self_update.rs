use anyhow::Result;

const REPO_OWNER: &str = "hiro-o918";
const REPO_NAME: &str = "skem";
const BIN_NAME: &str = "skem";

pub fn run_self_update() -> Result<()> {
    let current_version = env!("CARGO_PKG_VERSION");
    let target = self_update::get_target();

    let status = self_update::backends::github::Update::configure()
        .repo_owner(REPO_OWNER)
        .repo_name(REPO_NAME)
        .bin_name(BIN_NAME)
        .target(target)
        .current_version(current_version)
        .identifier(&format!("{BIN_NAME}-{target}.tar.gz"))
        .build()?
        .update()?;

    if status.updated() {
        println!(
            "Updated {BIN_NAME} from v{current_version} to {}",
            status.version()
        );
    } else {
        println!("{BIN_NAME} is already up to date (v{current_version})");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(REPO_OWNER, "hiro-o918");
        assert_eq!(REPO_NAME, "skem");
        assert_eq!(BIN_NAME, "skem");
    }

    #[test]
    fn test_update_builder_configuration() {
        let target = self_update::get_target();
        let current_version = env!("CARGO_PKG_VERSION");

        // Verify the builder can be configured without errors
        let result = self_update::backends::github::Update::configure()
            .repo_owner(REPO_OWNER)
            .repo_name(REPO_NAME)
            .bin_name(BIN_NAME)
            .target(target)
            .current_version(current_version)
            .identifier(&format!("{BIN_NAME}-{target}.tar.gz"))
            .build();

        assert!(
            result.is_ok(),
            "Update builder configuration should succeed"
        );
    }
}
