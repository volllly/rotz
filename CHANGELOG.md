# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.9.1] - 2022-10-30

### Added

- Added binaries to relases

## [0.9.0] - 2022-10-07

### Added

- Linked files are tracked and stored
- When a previously linked file is not a link target anymore it will be removed ([#8](https://github.com/volllly/rotz/issues/8))

### Changed

- When previously linked file is linked again it will be automatically overwritten without the need for the `--force` cli flag

## [0.8.1] - 2022-09-29

### Fixed

- Issue where rotz could not parse dots with mixed links section types ([#40](https://github.com/volllly/rotz/issues/40))

### Changed

- Updated cli parser to clap v4 which slightly changes help output

## [0.8.0] - 2022-09-16

### Added

- Template helpers `#windows`, `#linx` and `#darwin` which work like `if`s for the respective os
- `eval` template helper which evaluates the given string on the shell

## [0.7.1] - 2022-09-12

### Fixed

- Filtering of dots in commands was not working correctly

## [0.7.0] - 2022-09-11

### Changed

- The repo level config file now has support for a `force` key for forced values which cannot be changed by the config file
- Rotz can now automatically detect the filetype and parse the format if the feature (`yaml`, `toml` or `json`) is enabled
- The features `yaml`, `toml` and `json` can now be enabled simultaneously

### Added

- Added `whoami` variable to templating
- Added `directories` variable to templating
- Add ability to recurse into subdirectories

### Fixed

- Bug where the repo level config would not merge correctly

## [0.6.1] - 2022-08-18

### Changed

- The repo level config file now uses the key `global` instead of `default`
- The default `shell_command` on windows now correctly uses PowerShell instead of PowerShell Core

### Fixed

- The repo level config file can now override config default values

## [0.6.0] - 2022-07-29

### Added

- Implemented init command which initializes the config
- Added templating to `dot.(yaml|toml|json)` files

### Removed

- Removed the `repo` key from the config as its not needed

### Changed

- The `repo` argument is now required for the clone command

## [0.5.0] - 2022-07-15

### Added

- Implemented install command functionality

## [0.4.1] - 2022-06-30

### Fixed

- Wildcard "*" in install command not working 
- Defaults and global values in `dot.(yaml|toml|json)` files not working correctly

## [0.4.0] - 2022-06-29

### Added

- Global `--dry-run` cli parameter
- Implemented install command functionality
- Option to skip installing dependences in install command
- Option to continue on installation error in install command
- Support for a repo level config file. You can now add a `config.(yaml|toml|json)` file containing os specific defaults to the root of your dotfiles repo.
- `shell_command` configuration parameter

### Changed

- Improved Error messages

### Fixed

- Parsing of `dot.(yaml|toml|json)` files in the `installs` section

### Removed

- Removed the `update` command. Updates to the applications should be performed by your packagemanager.

## [0.3.2] - 2022-06-28

### Fixed

- Linking now also creates the parent directory if it's not present on windows

## [0.3.1] - 2022-05-27

### Added

- Added error codes and help messages

### Changed

- Refactored the command code

### Fixed

- Linking now also creates the parent directory if it's not present

 ## [0.3.0] - 2022-05-09

### Added

- `clone` command creates a config file with the repo configured if it does not exist
- Started adding unit tests

### Changed

- Better error messages
- Moved from [eyre](https://crates.io/crates/eyre) to [miette](https://crates.io/crates/miette) for error handline

## [0.2.0] - 2022-02-21

### Added

- Added `clone` command

### Fixed

- Fixed `link` command default value for Dots not working

## [0.1.1] - 2022-02-18

### Changed

- Updated Readme

## [0.1.0] - 2022-02-18

### Added

- Cli parsing
- Config parsing
- `yaml` support
- `toml` support
- `json` support
- Dotfile linking
- Error handling

[Unreleased]: https://github.com/volllly/rotz/compare/v0.9.0...HEAD
[0.9.0]: https://github.com/volllly/rotz/releases/tag/v0.9.0
[0.8.1]: https://github.com/volllly/rotz/releases/tag/v0.8.1
[0.8.0]: https://github.com/volllly/rotz/releases/tag/v0.8.0
[0.7.1]: https://github.com/volllly/rotz/releases/tag/v0.7.1
[0.7.0]: https://github.com/volllly/rotz/releases/tag/v0.7.0
[0.6.1]: https://github.com/volllly/rotz/releases/tag/v0.6.1
[0.6.0]: https://github.com/volllly/rotz/releases/tag/v0.6.0
[0.5.0]: https://github.com/volllly/rotz/releases/tag/v0.5.0
[0.4.1]: https://github.com/volllly/rotz/releases/tag/v0.4.1
[0.4.0]: https://github.com/volllly/rotz/releases/tag/v0.4.0
[0.3.2]: https://github.com/volllly/rotz/releases/tag/v0.3.2
[0.3.1]: https://github.com/volllly/rotz/releases/tag/v0.3.1
[0.3.0]: https://github.com/volllly/rotz/releases/tag/v0.3.0
[0.2.0]: https://github.com/volllly/rotz/releases/tag/v0.2.0
[0.1.1]: https://github.com/volllly/rotz/releases/tag/v0.1.1
[0.1.0]: https://github.com/volllly/rotz/releases/tag/v0.1.0
