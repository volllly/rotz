---
sidebar_position: 1
title: config.yaml
---

The following settings are configurable in the config file like so:

```yaml title="config.yaml"
dotfiles: <path to dotfiles>
link_type: <"symbolic"|"hard">
repo: <git repo url>
shell_command: <shell command template used for the install command>
variables: <map of variables which can be used in templates>
```

Those settings can be overridden in the cli when applicable (see `rotz --help` and `rotz <command> --help` to get more information).

## `shell_command`

This settig allows to specify how Rotz should launch the install command.

If this is not set the default values are used.

```yaml title="Windows"
shell_command: pwsh -NoProfile -C {{ quote "" cmd }}
```

```yaml title="Linux"
shell_command: bash -c {{ quote "" cmd }}
```

```yaml title="MacOS"
shell_command: zsh -c {{ quote "" cmd }}
```

## `variables`

These variables can be used in [templates](templating.md).

```yaml title="config.yaml"
variables:
  some: value
  array:
    - one
    - two
```

## Repo defaults

It is possible to put a config file in your repo conatining default values depending on the OS. These are overridden by the config file on the machine.

```yaml title=".dotfiles/config.yaml"
default:
  link_type: <globalDefault>

windows:
  dotfiles: <windowsDefault>

linux:
  dotfiles: <linuxDefault>

darwin:
  dotfiles: <macosDefault>
```