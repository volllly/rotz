---
layout: default
title: dot.yaml
nav_order: 2
permalink: /configuration/dot-yaml
parent: Configuration
---

# `dot.yaml`

1. TOC
{:toc}

The `dot.yaml` file consists of four optional keys:

| key        | requirement | function                                              |
|------------|-------------|-------------------------------------------------------|
| `links`    | `optional`  | Defines where to link which `dotfile`                 |
| `installs` | `optional`  | Defines the install command and install dependencies. |
| `updates`  | `optional`  | Defines the update command and update dependencies.   |
| `depends`  | `optional`  | Defines dependencies this application needs to work.  |

## `links`

The `links` section specifies where the dotfiles should be linked. **Command `Link-Dots`**

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

## `installs`

The `installs` section contains the install command and optional install dependencies. **Command `Install-Dots`**

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

## `updates`

The `updates` section contains the update command and optional update dependencies. **Command `Update-Dots`**

It works exactly like the `installs` key described above.

> ***Example:***
>
> *`nodejs/dot.yaml`*
> ```yaml
> ...
> updates:
>   cmd: scoop update nodejs
>   depends: [scoop]
> ```

## depends

The `depends` section contains an array of dependencies needed for the application to work correctly.

These dependencies will also be installed/updated when the application is installed/updated.

> ***Example:***
>
> *`zsh/dot.yaml`*
> ```yaml
> ...
> depends: [starship]
> ```