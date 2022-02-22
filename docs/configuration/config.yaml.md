---
layout: default
title: config.yaml
nav_order: 1
permalink: /configuration/config-yaml
parent: Configuration
---

# `config.yaml`

The following settings are configurable in the config file like so:

```yaml
dotfiles: <path to dotfiles>
link_type: <"symbolic"|"hard">
repo: <git repo url>
```

Those settings can be overridden in the cli when applicable (see `rotz --help` and `rotz <command> --help` to get more information).
