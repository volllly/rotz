---
title: OS Specific Configuration
sidebar_position: 4
---

You can specify different behaviors per OS in all configuration files.

Rotz can differentiate between Windows, Linux and MacOS.

To specify OS Specific behavior you need to add top level keys named `linux`, `windows`, `darwin` (for MacOS) and `general` (applied to all OSs).

```yaml title="Example: dots.yaml"
windows:
  installs:
    cmd: scoop install {{ name }}
    depends:
      - scoop
      - extras
darwin:
  installs:
    cmd: brew install {{ name }}
    depends:
      - brew
```
```yaml title="Example: neovim/dot.yaml"
windows:
  links:
    ginit.vim: ~\AppData\Local\nvim\ginit.vim
    init.vim: ~\AppData\Local\nvim\init.vim
    
global:
  links:
    ginit.vim: ~/.config/nvim/init.vim
    init.vim: ~/.config/nvim/ginit.vim
```

You can also combine multiple OSs per key separating them with a `|`.

```yaml title="Example: dots.yaml"
windows:
  installs:
    cmd: scoop install {{ name }}
    depends:
      - scoop
      - extras
darwin|linux:
  installs:
    cmd: brew install {{ name }}
    depends:
      - brew
```

