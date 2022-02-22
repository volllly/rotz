---
layout: default
title: Configuration
nav_order: 3
permalink: /configuration
has_children: true
---

# Configuration

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

The file `dot.yaml` contains information about how to install and update the application and where to link the dotfiles.
