---
title: Migration Guide
sidebar_position: 7
---

# Migration Guide: Single to Multiple Dotfiles Directories

This guide helps you migrate from a traditional single dotfiles directory to rotz's new multi-directory support introduced in v1.2.2.

## Why Migrate?

Multi-directory support offers several advantages:

- **Better Organization**: Separate personal, work, and system configurations
- **Conflict Resolution**: Explicit priority ordering when configs overlap
- **Modular Management**: Enable/disable configuration sets as needed
- **Repository Separation**: Keep different types of configs in separate git repositories

## Migration Strategies

### Strategy 1: Organizational Split

**Before:**
```
~/.dotfiles/
├── zsh/
├── git/
├── tmux/
├── vim/
├── work-ssh/
└── personal-ssh/
```

**After:**
```
~/.dotfiles/
├── personal/
│   ├── zsh/
│   ├── git/
│   ├── tmux/
│   ├── vim/
│   └── ssh/
└── work/
    ├── git/     # Work-specific git config
    └── ssh/     # Work SSH keys and config
```

**Configuration:**
```yaml
# Before
dotfiles: "~/.dotfiles"

# After  
dotfiles:
  - "~/.dotfiles/personal"
  - "~/.dotfiles/work"
```

### Strategy 2: Application-Based Split

**Before:**
```
~/.dotfiles/
├── bashrc/
├── zshrc/
├── vimrc/
├── tmux.conf/
├── gitconfig/
└── ssh_config/
```

**After:**
```
~/.dotfiles/
├── shell/
│   ├── bashrc/
│   └── zshrc/
├── editors/
│   └── vimrc/
├── tools/
│   ├── tmux.conf/
│   └── gitconfig/
└── system/
    └── ssh_config/
```

**Configuration:**
```yaml
# After
dotfiles:
  - "~/.dotfiles/shell"
  - "~/.dotfiles/editors" 
  - "~/.dotfiles/tools"
  - "~/.dotfiles/system"
```

### Strategy 3: Environment-Based Split

**Before:**
```
~/.dotfiles/
├── common-configs/
├── laptop-specific/
└── desktop-specific/
```

**After:**
```
~/.dotfiles/
├── common/      # Base configurations
├── laptop/      # Laptop-specific overrides
└── desktop/     # Desktop-specific overrides
```

**Configuration:**
```yaml
dotfiles:
  - "~/.dotfiles/common"   # Base configs
  - "~/.dotfiles/laptop"   # Environment-specific
```

## Step-by-Step Migration

### 1. Backup Your Current Setup

```bash
# Create a backup
cp -r ~/.dotfiles ~/.dotfiles.backup
```

### 2. Plan Your Directory Structure

Decide how you want to organize your dotfiles:
- By purpose (personal/work/system)
- By application (shell/editors/tools)
- By environment (common/laptop/desktop)
- Hybrid approach

### 3. Reorganize Your Files

```bash
# Example: Personal/Work split
cd ~/.dotfiles
mkdir personal work

# Move shared configs to personal (they'll have priority)
mv zsh personal/
mv vim personal/
mv tmux personal/

# Create work-specific configs
mv work-git work/git
mv work-ssh work/ssh
```

### 4. Update Your Configuration

```yaml
# ~/.config/rotz/config.yaml
dotfiles:
  - "~/.dotfiles/personal"  # Higher priority
  - "~/.dotfiles/work"      # Lower priority
link_type: symbolic
```

### 5. Test the Migration

```bash
# Test linking with dry-run first
rotz link --dry-run

# If everything looks good, do the actual linking
rotz link
```

### 6. Verify Everything Works

Check that all your configurations are properly linked and applications work as expected.

## Conflict Resolution

When the same configuration exists in multiple directories, rotz uses "first directory wins" resolution:

```yaml
dotfiles:
  - "~/.dotfiles/personal"  # This git config wins
  - "~/.dotfiles/work"      # This git config is ignored
```

Both directories have `git/dot.yaml`, but only the one in `personal/` will be used.

## Git Repository Management

### Single Repository Approach

Keep all directories in one git repository:

```bash
cd ~/.dotfiles
git add .
git commit -m "Reorganize into multiple directories"
```

### Multiple Repository Approach  

Split into separate repositories:

```bash
# Personal dotfiles
cd ~/.dotfiles/personal
git init
git remote add origin git@github.com:yourusername/personal-dotfiles.git

# Work dotfiles  
cd ~/.dotfiles/work
git init  
git remote add origin git@github.com:company/work-dotfiles.git
```

## Common Pitfalls

### 1. Incorrect Priority Order

**Problem:** Work configs override personal configs unexpectedly.

**Solution:** Ensure personal directory is listed first:
```yaml
dotfiles:
  - "~/.dotfiles/personal"  # First = highest priority
  - "~/.dotfiles/work"
```

### 2. Missing Dots After Migration

**Problem:** Some configurations don't appear after migration.

**Solution:** Verify dot.yaml files are in the correct locations and use glob patterns to debug:
```bash
rotz link --dry-run  # Shows which dots are found
```

### 3. Path Resolution Issues

**Problem:** Relative paths in dot configurations break.

**Solution:** Use absolute paths or ensure relative paths are correct from each directory.

## Rollback Plan

If migration doesn't work as expected:

1. Restore from backup:
   ```bash
   rm -rf ~/.dotfiles
   mv ~/.dotfiles.backup ~/.dotfiles
   ```

2. Revert configuration:
   ```yaml
   # ~/.config/rotz/config.yaml  
   dotfiles: "~/.dotfiles"
   ```

3. Re-link:
   ```bash
   rotz link
   ```

## Best Practices

- **Start Simple**: Begin with a basic personal/work split
- **Test Thoroughly**: Always use `--dry-run` first
- **Document Changes**: Keep notes about your directory structure
- **Gradual Migration**: Migrate one section at a time
- **Backup Everything**: Keep backups until you're confident in the new setup

## Getting Help

If you encounter issues during migration:

1. Check the [troubleshooting guide](../troubleshooting)
2. Use `rotz --help` and `rotz <command> --help`
3. Create an issue on [GitHub](https://github.com/volllly/rotz/issues)