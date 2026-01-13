---
title: Examples
sidebar_position: 6
---

You can see all of this functionality used in my [own dotfiles repository](https://github.com/volllly/.dotfiles).

## Multi-Dotfiles Configuration Examples

### Personal/Work Separation

```yaml
# config.yaml
dotfiles:
  - "~/.dotfiles/personal"
  - "~/.dotfiles/work"
link_type: symbolic
```

This setup allows you to:
- Keep personal configurations (shell, git, etc.) in `~/.dotfiles/personal`
- Keep work-specific configurations in `~/.dotfiles/work`
- Personal configs take priority in case of conflicts

### Modular Organization

```yaml
# config.yaml  
dotfiles:
  - "~/.dotfiles/core"      # Essential system configs
  - "~/.dotfiles/dev"       # Development tools
  - "~/.dotfiles/desktop"   # Desktop environment configs
link_type: symbolic
```

### Environment-Specific Setup

```yaml
# config.yaml
dotfiles:
  - "~/.dotfiles/common"    # Shared configs
  - "~/.dotfiles/laptop"    # Laptop-specific configs
  - "~/.dotfiles/work-vpn"  # Work VPN-specific configs
link_type: symbolic
```

### Migration Example

**Before (single directory):**
```yaml
dotfiles: "~/.dotfiles"
link_type: symbolic
```

**After (organized into multiple directories):**
```yaml
dotfiles:
  - "~/.dotfiles/shell"     # bash, zsh, fish configs
  - "~/.dotfiles/editors"   # vim, emacs, vscode
  - "~/.dotfiles/tools"     # git, tmux, etc.
link_type: symbolic
```

### Directory Structure Example

```
~/.dotfiles/
├── personal/
│   ├── zsh/
│   │   └── dot.yaml
│   ├── git/
│   │   └── dot.yaml
│   └── tmux/
│       └── dot.yaml
├── work/
│   ├── git/
│   │   └── dot.yaml      # Work-specific git config
│   └── ssh/
│       └── dot.yaml
└── system/
    ├── fontconfig/
    │   └── dot.yaml
    └── systemd/
        └── dot.yaml
```

In this example:
- Personal git config in `personal/git/` takes priority over work git config
- Work-specific ssh configuration is isolated
- System-level configurations are separate from user configs
