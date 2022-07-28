---
sidebar_position: 3
title: Defaults
---

The repo can also contain a default file `dots.yaml` in the root folder of the repo.

This file contains defaults which are automatically used for empty keys in the `dot.yaml` files.

You can use template strings (`{{ name }}`) to substitute the name of the application (the name of the folder the `dot.yaml` file is located in).

```yaml title="Example: dots.yaml"
installs:
  cmd: scoop install {{ name }}
  depends:
    - scoop
    - extras
```

:::caution

In the 1.0 release [advanced templating](templating.md) will also become available in this file.
In the future you will then need to escape the `name` variable since it is not yet available when parsing the defaults.

This will look like this
```yaml title="Example: dots.yaml"
installs:
  # `name` will be substitutet when installing the dot and use the dot name
  cmd: scoop install \{{ name }}
  depends:
# this will be substituted when parsing the defaults
{{#each config.variables.dendencies }}
  - {{this}}
{{/each }}
```

:::