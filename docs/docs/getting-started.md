---
sidebar_position: 1
title: Getting Started
---

## Overview

Rotz has three main functionalities:

1. Linking dotfiles from a common repository to your system
2. Installing the applications you need to start working on an new/empty machine
3. Full Cross platform functionality [See Configuration](#os-specific-configuration)

## Installation

### Homebrew

On Linux and MacOS you can install Rotz using [Homebrew](https://brew.sh/).

```sh title="homebrew"
brew install volllly/tap/rotz
```

### Scoop

On Windows you can install Rotz using [Scoop](https://scoop.sh/).

```pwsh title="scoop"
scoop bucket add volllly https://github.com/volllly/scoop-bucket
scoop install volllly/rotz
```

### Cargo

You can install Rotz using cargo everywhere if Rust is installed.

```bash title="cargo"
cargo install rotz
```

#### Other File Formats

Rotz uses [`yaml`](https://yaml.org/), [`toml`](https://toml.io/) or [`json`](https://www.json.org/) configuration files per default.

You can install Rotz with support for only one of the filetypes by using the `--features` flag.

```bash title="toml"
cargo install rotz --no-default-features --features toml
```

```bash title="json"
cargo install rotz --no-default-features --features json
```

## Installer scripts

```sh title="Linux and MacOS"
curl -fsSL volllly.github.io/rotz/install.sh | sh
```

```pwsh title="Windows"
irm volllly.github.io/rotz/install.sh | iex
```

---

## Getting Started

If you already have a `dotfiles` repo you can clone it with the `rotz clone` command.

```sh title="Clone command"
rotz clone git@github.com:<user>/<repo>.git
```

To bootstrap your dev environment use `rotz install`.

To link your `dotfiles` use `rotz link`.