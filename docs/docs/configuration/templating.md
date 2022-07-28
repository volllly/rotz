---
title: dot.yaml
sidebar_position: 5
---

You can use [handlebars](https://handlebarsjs.com/guide/) template syntax in `dot.yaml` files.

This allows for e.g. access to environment variables.

## Parameters

| Variable | description                                            | example                                                                 |
| -------- | ------------------------------------------------------ | ----------------------------------------------------------------------- |
| `config` | The current config.                                    | `depends: [ {{#each config.variables.some ~}} "{{this}}", {{/each }} ]` |
| `env`    | A map of Environment variables                         | `some.file: {{ env.HOME }}/some.file`                                   |
| `name`   | The name of the current dot.                           | `installs: apt install {{ name }}`                                      |
| `os`     | The current os (either `windows`, `linux` or `darwin`) | `{{#if (eq os "windows")}}some: value{{/if}}`                           |

:::caution

At the moment this is only available in the `dots.yaml` files. In the 1.0 release this will also become available in the [defaults](defaults.md).

:::