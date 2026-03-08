# How It Works

1. Reads `.skem.yaml` configuration
2. For each dependency (processed in parallel):
   - Checks remote commit SHA using `git ls-remote`
   - Compares with `.skem.lock` to detect changes
   - If changed, downloads files using sparse checkout
   - Strips path prefixes and copies to output directory
   - Runs per-dependency hooks if specified
   - Updates lockfile on success
3. Runs global `post_hooks` after all dependencies are processed
