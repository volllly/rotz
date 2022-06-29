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
