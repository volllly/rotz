---
layout: default
title: Home
nav_order: 1
description: "Fully cross platform dotfile manager and dev environment bootstrapper written in Rust."
permalink: /
---

# Fully cross platform dotfile manager and dev environment bootstrapper written in Rust

> ```
> Rust Dotfilemanager
> Rust Dotfile manager
> Rust Dotfile s
> Rust Dot s
> R ust Dots
> R ots
> Rot s
> ```

Rotz is an evolution of [Dotted](https://github.com/volllly/Dotted).

---

## Status

This project is still in development.

Linking dotfiles on windows already works but is not very well tested.

Expect more features in the next release which should be ready in a few weeks.

## [Roadmap](https://github.com/users/volllly/projects/1/views/1)

## Overview

Rotz has three main functionalities:

1. Linking dotfiles from a common repository to your system
2. Installing the applications you need to start working on an new/empty machine *`[in development]`*
3. Full Cross platform functionality [See Configuration](#os-specific-configuration)

## Installation

You can install rotz using cargo.

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

---

## Getting Started

If you already have a `dotfiles` repo you can clone it with the `rotz clone` command.

To bootstrap your dev environment use `rotz install`. *`[in development]`*

To link your `dotfiles` use `rotz link`.