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

You can install Rotz using cargo.

```bash
cargo install rotz
```

### Other File Formats

Rotz uses [`yaml`](https://yaml.org/) configuration files per default. You can also use [`toml`](https://toml.io/) or [`json`](https://www.json.org/) files instead.

To use another format install Rotz using one of the following comands:
```bash title="toml"
cargo install rotz --no-default-features --features toml
```

```bash title="json"
cargo install rotz --no-default-features --features json
```

---

## Getting Started

If you already have a `dotfiles` repo you can clone it with the `rotz clone` command.

```sh title="Clone command"
rotz clone git@github.com:<user>/<repo>.git
```

To bootstrap your dev environment use `rotz install`.

To link your `dotfiles` use `rotz link`.