---
title: Templating
sidebar_position: 5
---

You can use [handlebars](https://handlebarsjs.com/guide/) template syntax in [`dot.yaml`](dot.yaml.mdx) files and the [defaults file](defaults.md).

This allows for e.g. access to environment variables.

## Parameters

| Variable      | description                                                                                                                       | example                                                                 |
| ------------- | --------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------- |
| `config`      | The current config                                                                                                                | `depends: [ {{#each config.variables.some ~}} "{{this}}", {{/each }} ]` |
| `env`         | A map of Environment variables                                                                                                    | `some.file: {{ env.HOME }}/some.file`                                   |
| `name`        | The name of the current dot                                                                                                       | `installs: apt install {{ name }}`                                      |
| `os`          | The current os (either `windows`, `linux` or `darwin`)                                                                            | `{{#if (eq os "windows")}}some: value{{/if}}`                           |
| `whoami`      | A map of information about the environment as provided by the [whoami](https://github.com/dirs-dev/directories-rs#features) crate | `some.file: /home/{{ whoami.username }}/some.file`                      |
| `directories` | A map of directories as provided by the [directories](https://github.com/ardaku/whoami#features) crate                            | `some.file: {{ directories.home }}/some.file`                           |
