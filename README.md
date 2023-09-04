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

## [🗺️ Roadmap](https://github.com/users/volllly/projects/1/views/1)

## [📖 Documentation](https://volllly.github.io/rotz/)

## Overview

Rotz has three main functionalities:

1. Linking dotfiles from a common repository to your system
2. Installing the applications you need to start working on an new/empty machine
3. Full Cross platform functionality [See Configuration](https://volllly.github.io/rotz/docs/configuration/os-specific-configuration)

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


## Getting Started

If you already have a `dotfiles` repo you can clone it with the `rotz clone` command.

```sh
rotz clone git@github.com:<user>/<repo>.git
```

To bootstrap your dev environment use `rotz install`.

To link your `dotfiles` use `rotz link`.

## Usage

Run `rotz --help` to see all commands Rotz has.

## Contribute

Feel free to create pull requests and issues for bugs, features or questions. 
