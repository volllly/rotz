---
title: Templating
sidebar_position: 5
---

You can use [handlebars](https://handlebarsjs.com/guide/) template syntax in [`dot.yaml`](dot.yaml.mdx) files and the [defaults file](defaults.mdx).

This allows for e.g. access to environment variables.

## Variables

| Variable | Description                                                                                                                                                                     | Example                                                                 |
|----------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|-------------------------------------------------------------------------|
| `config` | The current config                                                                                                                                                              | `depends: [ {{#each config.variables.some ~}} "{{this}}", {{/each }} ]` |
| `env`    | A map of Environment variables                                                                                                                                                  | `some.file: {{ env.HOME }}/some.file`                                   |
| `name`   | The name of the current dot                                                                                                                                                     | `installs: apt install {{ name }}`                                      |
| `os`     | The current os (either `windows`, `linux` or `darwin`) as used in dots                                                                                                          | `{{#if (eq os "windows")}}some: value{{/if}}`                           |
| `whoami` | A map of information about the environment (see [whoami](#whoami)). Provided by the [whoami](https://github.com/ardaku/whoami#features) crate.                                  | `some.file: /home/{{ whoami.username }}/some.file`                      |
| `dirs`   | A map of directories (see [below](#directories) for a list of available directories). Provided by the [directories](https://github.com/dirs-dev/directories-rs#features) crate. | `some.file: {{ dirs.base.home }}/some.file`                             |

### `whoami`

| Variable             | Description                               |
|----------------------|-------------------------------------------|
| `whoami.desktop_env` | Information about the Desktop environment |
| `whoami.devicename`  | The device name                           |
| `whoami.distro`      | The os distro                             |
| `whoami.hostname`    | The hostname                              |
| `whoami.lang`        | An array of the users prefered languages  |
| `whoami.platform`    | The current platform                      |
| `whoami.realname`    | The users full name                       |
| `whoami.username`    | The current users username                |
| `whoami.arch`        | The CPU architecture of the system.       |

### `directories`

| Variable          |
|-------------------|
| `dirs.base.cache`      |
| `dirs.base.config`     |
| `dirs.base.data`       |
| `dirs.base.data_local` |
| `dirs.base.home`       |
| `dirs.base.preference` |
|                   |
| `dirs.user.audio`      |
| `dirs.user.desktop`    |
| `dirs.user.document`   |
| `dirs.user.download`   |
| `dirs.user.home`       |
| `dirs.user.picture`    |
| `dirs.user.public`     |
| `dirs.user.template`   |
| `dirs.user.video`      |

## Helpers

Rotz comes with helpers provided by the [handlebars_misc_helpers](https://github.com/davidb/handlebars_misc_helpers) crate.

Theres also a number of inbuilt helpers provided

### `#windows`, `#linx` and `#darwin`

These helpers are shorthands for checking the curent os.

Instea of `{{ #if (eq os "windows") }}{{ else }}{{ /if }}` they can be used like this `{{ #windows }}{{ else }}{{ /windows }}`.

### `eval`

The eval helper can be used to evalate a string on the shell configured by [`shell_command`](config.yaml.mdx#shell_command).

The helper can be used like this `{{ eval "some --shell command" }}`
