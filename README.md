# Rotz 👃
[![crates.io](https://img.shields.io/crates/v/rotz)](https://crates.io/crates/rotz)
![](https://img.shields.io/badge/platform-windows%20%7C%20linux%20%7C%20macos-lightgrey)
[![](https://img.shields.io/crates/l/rotz)](https://github.com/volllly/rotz/blob/main/LICENSE)

Fully cross platform dotfile manager and dev environment bootstrapper written in Rust.

> `Rust Dotfilemanager`<br>
> `Rust Dotfile manager`<br>
> `Rust Dotfile s`<br>
> `Rust Dot s`<br>
> `R ust Dots`<br>
> `R ots`<br>
> `Rot s`<br>
> `Rotz`

## [📖 Documentation](https://volllly.github.io/rotz/)

## Overview

Rotz has three main functionalities:

1. Linking dotfiles from a common repository to your system
2. Installing the applications you need to start working on an new/empty machine
3. Full Cross platform functionality [See Configuration](https://volllly.github.io/rotz/docs/configuration/os-specific-configuration)

### Multi-Dotfiles Support (New in v1.2.2)

Rotz now supports managing dotfiles from multiple directories, enabling better organization and separation of concerns:

- **Personal/Work separation**: Keep personal and work dotfiles in separate repositories
- **Modular organization**: Organize by application type (shell, editors, system tools)  
- **Environment-specific configs**: Different configs for different environments

```yaml
# config.yaml
dotfiles:
  - "~/.dotfiles/personal"
  - "~/.dotfiles/work"
  - "~/.dotfiles/system"
```

When using multiple directories, rotz applies "first directory wins" conflict resolution and tracks which directory each dot comes from during linking.

## Installation

### Homebrew

On Linux and MacOS you can install Rotz using [Homebrew](https://brew.sh/).

```sh
brew install volllly/tap/rotz
```

### Scoop

On Windows you can install Rotz using [Scoop](https://scoop.sh/).

```pwsh
scoop bucket add volllly https://github.com/volllly/scoop-bucket
scoop install volllly/rotz
```

### Cargo

You can install Rotz using cargo everywhere if Rust is installed.

```bash
cargo install rotz
```

#### File Formats

Rotz uses [`yaml`](https://yaml.org/), [`toml`](https://toml.io/) or [`json`](https://www.json.org/) configuration files per default.

> ***Note:** Rotz will auto detect the correct filetype.*

You can install rotz with support for only one of the filetypes by using the `--features` flag.
* ```sh
  cargo install rotz --no-default-features --features toml
  ```
* ```sh
  cargo install rotz --no-default-features --features json
  ```

## Installer scripts

```sh
curl -fsSL volllly.github.io/rotz/install.sh | sh
```

```pwsh
irm volllly.github.io/rotz/install.ps1 | iex
```

## Getting Started

If you already have a `dotfiles` repo you can clone it with the `rotz clone` command.

```sh
rotz clone git@github.com:<user>/<repo>.git
```

To bootstrap your dev environment use `rotz install`.

To link your `dotfiles` use `rotz link`.

## Usage

Run `rotz --help` to see all commands Rotz has.

### Configuration Examples

#### Single Directory (Traditional)
```yaml
# ~/.config/rotz/config.yaml
dotfiles: "~/.dotfiles"
link_type: symbolic
```

#### Multiple Directories
```yaml
# ~/.config/rotz/config.yaml
dotfiles:
  - "~/.dotfiles/personal"  # Personal configs (highest priority)
  - "~/.dotfiles/work"      # Work-specific configs
  - "~/.dotfiles/system"    # System-level configs
link_type: symbolic
```

Existing single-directory configurations remain fully compatible.

## Contribute

Feel free to create pull requests and issues for bugs, features or questions. 
