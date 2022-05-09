# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]


 ## [v0.3.0] - 2022-05-09

### Added

- `clone` command creates a config file with the repo configured if it does not exist
- Started adding unit tests

### Changed

- Better error messages
- Moved from [eyre](https://crates.io/crates/eyre) to [miette](https://crates.io/crates/miette) for error handline

## [v0.2.0] - 2022-02-21

### Added

- Added `clone` command

### Fixed

- Fixed `link` command default value for Dots not working

## [v0.1.1] - 2022-02-18

### Changed

- Updated Readme

## [v0.1.0] - 2022-02-18

### Added

- Cli parsing
- Config parsing
- `yaml` support
- `toml` support
- `json` support
- Dotfile linking
- Error handling

[Unreleased]: https://github.com/volllly/rotz/compare/v0.3.0...HEAD
[v0.3.0]: https://github.com/volllly/rotz/releases/tag/v0.3.0
[v0.2.0]: https://github.com/volllly/rotz/releases/tag/v0.2.0
[v0.1.1]: https://github.com/volllly/rotz/releases/tag/v0.1.1
[v0.1.0]: https://github.com/volllly/rotz/releases/tag/v0.1.0
