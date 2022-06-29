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

Rotz is an evolution of [Dotted](https://github.com/volllly/Dotted).

## Status

This project is still in development.

Expect more features in the next release.

## [Roadmap](https://github.com/users/volllly/projects/1/views/1)

## [Documentation](https://volllly.github.io/rotz/) *`[in development]`*

## Overview

Rotz has three main functionalities:

1. Linking dotfiles from a common repository to your system
2. Installing the applications you need to start working on an new/empty machine
3. Full Cross platform functionality [See Configuration](#os-specific-configuration)

## Installation

You can install Rotz using cargo.

```sh
cargo install rotz
```

### Other File Formats

Rotz uses [`yaml`](https://yaml.org/) configuration files per default. You can also use [`toml`](https://toml.io/) or [`json`](https://www.json.org/) files instead.

To use another format install Rotz using one of the following comands:
* ```sh
  cargo install rotz --no-default-features --features toml
  ```
* ```sh
  cargo install rotz --no-default-features --features json
  ```

## Getting Started

If you already have a `dotfiles` repo you can clone it with the `rotz clone` command.

To bootstrap your dev environment use `rotz install`. *`[in development]`*

To link your `dotfiles` use `rotz link`.

---
## Usage

Run `rotz --help` to see all commands Rotz has.

## Configuration

Rotz uses a git repo containing the`dotfiles` and [`yaml`](https://yaml.org/) files for configuration.

> ***Note:** To use another file format see [Other File Formats](#other-file-formats).*

This git repo should be located at `~/.dotfiles`. Different paths can be specified using the `--dotfiles` cli flag or in the Rotz [config file](#configyaml).

> ***Note:** The location of the config file can be overridden using the `--config` cli flag. To get the default location of the config run `rotz --help`*

Each managed application has a subfolder containing its `dotfiles` and a `dot.yaml` file.

> ***Example:***
> ```
> └── vscode
>     ├── dot.yaml
>     ├── keybindings.json
>     └── settings.json
> ```

The file `dot.yaml` contains information about how to install the application and where to link the dotfiles.

## `config.yaml`

The following settings are configurable in the config file like so:

```yaml
dotfiles: <path to dotfiles>
link_type: <"symbolic"|"hard">
repo: <git repo url>
```

Those settings can be overridden in the cli when applicable (see `rotz --help` and `rotz <command> --help` to get more information).

## `dot.yaml`

The `dot.yaml` file consists of four optional keys:

| key        | requirement | function                                              |
|------------|-------------|-------------------------------------------------------|
| `links`    | `optional`  | Defines where to link which `dotfile`                 |
| `installs` | `optional`  | Defines the install command and install dependencies. |
| `depends`  | `optional`  | Defines dependencies this application needs to work.  |

### `links`

The `links` section specifies where the dotfiles should be linked.

It consists of multiple `key: value` pairs where the `key` is the filename of the `dotfile` and the `value` is the link path.

> ***Example:***
>
> *`vscode/dot.yaml`*
> ```yaml
> ...
> links:
>   keybindings.json: ~\AppData\Roaming\Code\User\keybindings.json
>   settings.json: ~\AppData\Roaming\Code\User\settings.json
> ```

### `installs`

The `installs` section contains the install command and optional install dependencies.

It can either be a `string` containing the install command or have two sub keys.

| key       | requirement | function                           |
|-----------|-------------|------------------------------------|
| `cmd`     | `required`  | Contains the install command.      |
| `depends` | `optional`  | Contains an array of dependencies. |

> ***Examples:***
>
> *`nodejs/dot.yaml`*
> ```yaml
> ...
> installs:
>   cmd: scoop install nodejs
>   depends: [scoop]
> ```
> *`scoop/dot.yaml`*
>
> ```yaml
> ...
> installs: iex (new-object net.webclient).downloadstring('https://get.scoop.sh')
> ```

### depends

The `depends` section contains an array of dependencies needed for the application to work correctly.

These dependencies will also be installed when the application is installed.

> ***Example:***
>
> *`zsh/dot.yaml`*
> ```yaml
> ...
> depends: [starship]
> ```

## Defaults

The repo can also contain a default file `dots.yaml` in the root folder of the repo.

This file contains defaults which are automatically used for empty keys in the `dot.yaml` files.

You can use template strings (`{{ name }}`) to substitute the name of the application (the name of the folder the `dot.yaml` file is located in).

> ***Example:***
>
> *`dots.yaml`*
> ```yaml
> installs:
>   cmd: scoop install {{ name }}
>   depends:
>     - scoop
>     - extras
> ```

## OS Specific Configuration

You can specify different behaviors per OS in all configuration files.

Rotz can differentiate between Windows, Linux and MacOS.

To specify OS Specific behavior you need to add top level keys named `linux`, `windows`, `darwin` (for MacOS) and `general` (applied to all OSs).

> ***Examples:***
>
> *`dots.yaml`*
> ```yaml
> windows:
>   installs:
>     cmd: scoop install {{ name }}
>     depends:
>       - scoop
>       - extras
> darwin:
>   installs:
>     cmd: brew install {{ name }}
>     depends:
>       - brew
> ```
> *`neovim/dot.yaml`*
> ```yaml
> windows:
>   links:
>     ginit.vim: ~\AppData\Local\nvim\ginit.vim
>     init.vim: ~\AppData\Local\nvim\init.vim
>     
> global:
>   links:
>     ginit.vim: ~/.config/nvim/init.vim
>     init.vim: ~/.config/nvim/ginit.vim
> ```

You can also combine multiple OSs per key separating them with a `|`.

> ***Example:***
>
> *`dots.yaml`*
> ```yaml
> windows:
>   installs:
>     cmd: scoop install {{ name }}
>     depends:
>       - scoop
>       - extras
> darwin|linux:
>   installs:
>     cmd: brew install {{ name }}
>     depends:
>       - brew
> ```


## Example Repository

You can see all of this functionality used in my [own dotfiles repository](https://github.com/volllly/.dotfiles).

## Contribute

Feel free to create pull requests and issues for bugs, features or questions. 