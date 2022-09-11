---
title: Templating
sidebar_position: 5
---

You can use [handlebars](https://handlebarsjs.com/guide/) template syntax in [`dot.yaml`](dot.yaml.mdx) files and the [defaults file](defaults.mdx).

This allows for e.g. access to environment variables.

## Variables

| Variable      | Description                                                                                                                                              | Example                                                                 |
| ------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------- |
| `config`      | The current config                                                                                                                                       | `depends: [ {{#each config.variables.some ~}} "{{this}}", {{/each }} ]` |
| `env`         | A map of Environment variables                                                                                                                           | `some.file: {{ env.HOME }}/some.file`                                   |
| `name`        | The name of the current dot                                                                                                                              | `installs: apt install {{ name }}`                                      |
| `os`          | The current os (either `windows`, `linux` or `darwin`) as used in dots                                                                                   | `{{#if (eq os "windows")}}some: value{{/if}}`                           |
| `whoami`      | A map of information about the environment (see [whoami](#whoami)). Provided by the [whoami](https://github.com/dirs-dev/directories-rs#features) crate. | `some.file: /home/{{ whoami.username }}/some.file`                      |
| `directories` | A map of directories (see [directories](#directories)). Provided by the [directories](https://github.com/ardaku/whoami#features) crate                   | `some.file: {{ directories.home }}/some.file`                           |
 
## `whoami`

| Variable      | Description                               |
| ------------- | ----------------------------------------- |
| `desktop_env` | Information about the Desktop environment |
| `devicename`  | The device name                           |
| `distro`      | The os distro                             |
| `hostname`    | The hostname                              |
| `lang`        | An array of the users prefered languages  |
| `platform`    | The current platform                      |
| `realname`    | The users full name                       |
| `username`    | The current users username                |

## `directories`

| Group  | Variable     |
| ------ | ------------ |
| `base` | `cache`      |
| `base` | `config`     |
| `base` | `data`       |
| `base` | `data_local` |
| `base` | `home`       |
| `base` | `preference` |
|        |              |
| `user` | `audio`      |
| `user` | `desktop`    |
| `user` | `document`   |
| `user` | `download`   |
| `user` | `home`       |
| `user` | `picture`    |
| `user` | `public`     |
| `user` | `template`   |
| `user` | `video`      |