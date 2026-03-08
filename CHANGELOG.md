# Changelog

## [0.2.0](https://github.com/hiro-o918/skem/compare/v0.1.0...v0.2.0) (2026-03-08)


### Features

* Add self-update command ([#23](https://github.com/hiro-o918/skem/issues/23)) ([f07bd63](https://github.com/hiro-o918/skem/commit/f07bd632cd71a4c802638446a9a7c7b9433a2819))
* Execute post_hooks after all dependencies are synced ([#31](https://github.com/hiro-o918/skem/issues/31)) ([07dbe0e](https://github.com/hiro-o918/skem/commit/07dbe0e4b3e44bf6a3880f1ae96882b7505f894c))
* Pass SKEM_SYNCED_FILES env var to hooks ([#30](https://github.com/hiro-o918/skem/issues/30)) ([2b6be0a](https://github.com/hiro-o918/skem/commit/2b6be0afd8b7f27a6ad3498ac6c1e39bdbd0ac33))


### Bug Fixes

* **deps:** update rust crate schemars to v1 ([#27](https://github.com/hiro-o918/skem/issues/27)) ([058aa23](https://github.com/hiro-o918/skem/commit/058aa239063d4a257a00a66974b35a4ed2a02071))
* **deps:** update rust minor-major dependencies to 0.9 ([#26](https://github.com/hiro-o918/skem/issues/26)) ([4035b65](https://github.com/hiro-o918/skem/commit/4035b650d6d0caaf188a8f1214c32a56adec1223))
* Error when copy_files matches zero files during sync ([#25](https://github.com/hiro-o918/skem/issues/25)) ([1d536c4](https://github.com/hiro-o918/skem/commit/1d536c4bb74cd2fd4bf6d808afbe98242021e618)), closes [#22](https://github.com/hiro-o918/skem/issues/22)


### Documentation

* Fix environment variable passing in install examples ([#19](https://github.com/hiro-o918/skem/issues/19)) ([3c07a45](https://github.com/hiro-o918/skem/commit/3c07a4534162544f55b86d64e973a85f4f69346c))


### Miscellaneous

* Add Renovate configuration ([#21](https://github.com/hiro-o918/skem/issues/21)) ([dba5ff9](https://github.com/hiro-o918/skem/commit/dba5ff9caf11218dfe479ef33315dbf439824dcb))

## 0.1.0 (2026-03-08)


### Features

* Add add/rm/ls/check subcommands and make rev optional ([#10](https://github.com/hiro-o918/skem/issues/10)) ([ce1488e](https://github.com/hiro-o918/skem/commit/ce1488e4b38ed4ba8686eefc192a1f56f5d281ce))
* Add config validation for check and sync commands ([#14](https://github.com/hiro-o918/skem/issues/14)) ([f139ce9](https://github.com/hiro-o918/skem/commit/f139ce91d23fdcae047c108979e026a88deb7f63))
* Add interactive mode for add command ([#13](https://github.com/hiro-o918/skem/issues/13)) ([92d27d3](https://github.com/hiro-o918/skem/commit/92d27d37cef1d8570bd278ba698e3d463e0f48e5))
* Add post_hooks field to Config for global post-sync hooks ([#12](https://github.com/hiro-o918/skem/issues/12)) ([010f07a](https://github.com/hiro-o918/skem/commit/010f07a80a38406426a683e9735b8acb2ffca68d))
* Add repo and rev fields to LockEntry ([#11](https://github.com/hiro-o918/skem/issues/11)) ([6a9f786](https://github.com/hiro-o918/skem/commit/6a9f786598c2aa4524353896504e716ef7ec363b))
* Define core data models for Config and Lockfile ([32f7ebb](https://github.com/hiro-o918/skem/commit/32f7ebb0ba9323235ec41ceaebac5da4b560302b))
* Define core data models for Config and Lockfile ([3b8757d](https://github.com/hiro-o918/skem/commit/3b8757d75a8ca62713df875cbae0798d87533971))
* Implement 'skem init' command ([ceb5d5a](https://github.com/hiro-o918/skem/commit/ceb5d5a043c3f0de7fe56597495d1dc904e4d10e))
* Implement 'skem init' command ([4dd6c68](https://github.com/hiro-o918/skem/commit/4dd6c689575c196b9491bafb3e3b875c9e275999))
* Implement 'skem schema' command ([#4](https://github.com/hiro-o918/skem/issues/4)) ([810cdcc](https://github.com/hiro-o918/skem/commit/810cdcc19b8e45f6645b7780a99eb58c66f8213c))
* Implement change detection using git ls-remote and lockfile ([#6](https://github.com/hiro-o918/skem/issues/6)) ([bcdbf20](https://github.com/hiro-o918/skem/commit/bcdbf201bd87282dc44e985d864535c350d93e60))
* Implement CLI structure with clap ([f2ed3fc](https://github.com/hiro-o918/skem/commit/f2ed3fcbbd5a34bcb2a8f2da8b13d976f8b2afd6))
* Implement CLI structure with clap ([ff5a973](https://github.com/hiro-o918/skem/commit/ff5a973ad473664f773e6fe2a2cfffea9265b3d5))
* Implement core synchronization workflow ([#9](https://github.com/hiro-o918/skem/issues/9)) ([e6f5da0](https://github.com/hiro-o918/skem/commit/e6f5da00c99fdfc6b746a4bb4e288a0cac66135a))
* Implement Git command wrapper using std::process::Command ([#5](https://github.com/hiro-o918/skem/issues/5)) ([1aa2c3a](https://github.com/hiro-o918/skem/commit/1aa2c3aa6c4f36236be3f2125c0f2e6a72142cff))
* Implement path stripping and file copying to output directory ([#8](https://github.com/hiro-o918/skem/issues/8)) ([171ad3d](https://github.com/hiro-o918/skem/commit/171ad3d0466233e00a212945d9a34a1ffb27ef50))
* Implement sparse checkout and file download ([#7](https://github.com/hiro-o918/skem/issues/7)) ([36a74f6](https://github.com/hiro-o918/skem/commit/36a74f6cf7502216a8b7f14056bb1a168ec05b3a))


### Documentation

* Update README and show help when no subcommand is provided ([#15](https://github.com/hiro-o918/skem/issues/15)) ([28ec407](https://github.com/hiro-o918/skem/commit/28ec4073cebefc34d5271158b4ece3ed98d73850))


### Miscellaneous

* Initialize Rust project with CI/CD workflows ([e19adad](https://github.com/hiro-o918/skem/commit/e19adade9cfa9957d5483857d13cd9785469da12))
* Prepare for 0.1.0 release ([#18](https://github.com/hiro-o918/skem/issues/18)) ([1806282](https://github.com/hiro-o918/skem/commit/18062824e500553222ba8ac9c5c945b1324e35a0))
