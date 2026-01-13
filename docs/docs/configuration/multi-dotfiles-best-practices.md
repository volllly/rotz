---
title: Multi-Dotfiles Best Practices
sidebar_position: 8
---

# Multi-Dotfiles Best Practices

This guide provides best practices for organizing and managing multiple dotfiles directories effectively with rotz.

## Directory Organization Strategies

### 1. Purpose-Based Organization

Organize directories by their intended purpose or scope:

```yaml
dotfiles:
  - "~/.dotfiles/personal"    # Personal configurations
  - "~/.dotfiles/work"        # Work-specific configurations  
  - "~/.dotfiles/system"      # System-level configurations
```

**Benefits:**
- Clear separation of concerns
- Easy to enable/disable entire contexts
- Natural backup and sync boundaries

**Use Cases:**
- Separate personal and professional environments
- Different configurations for different roles
- Isolate sensitive work configurations

### 2. Application-Based Organization

Group configurations by the applications they configure:

```yaml
dotfiles:
  - "~/.dotfiles/shell"       # bash, zsh, fish
  - "~/.dotfiles/editors"     # vim, emacs, vscode
  - "~/.dotfiles/terminals"   # tmux, screen, alacritty
  - "~/.dotfiles/tools"       # git, docker, kubectl
```

**Benefits:**
- Logical grouping by functionality
- Easy to maintain related configurations together
- Simple to share specific tool configurations

**Use Cases:**
- Large dotfiles collections
- Modular configuration sharing
- Tool-specific development teams

### 3. Environment-Based Organization

Structure directories by deployment environment:

```yaml
dotfiles:
  - "~/.dotfiles/common"      # Base configurations
  - "~/.dotfiles/laptop"      # Laptop-specific
  - "~/.dotfiles/server"      # Server environment
  - "~/.dotfiles/container"   # Container-specific
```

**Benefits:**
- Environment-specific customizations
- Inheritance-like behavior (common + specific)
- Easy environment switching

**Use Cases:**
- Multiple development environments
- Different hardware configurations
- Container vs. native deployments

## Priority and Conflict Resolution

### Understanding Priority Order

Rotz processes directories in order, with earlier directories having higher priority:

```yaml
dotfiles:
  - "~/.dotfiles/priority-1"  # Highest priority
  - "~/.dotfiles/priority-2"  # Medium priority
  - "~/.dotfiles/priority-3"  # Lowest priority
```

### Strategic Priority Planning

**High Priority (First):**
- Personal or user-specific configurations
- Environment-specific overrides
- Local customizations

**Medium Priority (Middle):**
- Team or project configurations
- Default organizational settings
- Shared configurations

**Low Priority (Last):**
- Base or fallback configurations
- System defaults
- Vendor-provided configurations

### Example Priority Structure

```yaml
# Recommended priority order
dotfiles:
  - "~/.dotfiles/local"       # Machine-specific overrides
  - "~/.dotfiles/personal"    # Personal preferences  
  - "~/.dotfiles/project"     # Current project configs
  - "~/.dotfiles/team"        # Team standards
  - "~/.dotfiles/base"        # Base configurations
```

## Repository Management Strategies

### Single Repository with Subdirectories

**Structure:**
```
~/.dotfiles/
├── .git/
├── personal/
│   ├── zsh/dot.yaml
│   └── git/dot.yaml
├── work/
│   ├── git/dot.yaml
│   └── ssh/dot.yaml
└── system/
    └── fonts/dot.yaml
```

**Pros:**
- Simple git management
- Easy to clone entire setup
- Unified versioning

**Cons:**
- All configurations in one repository
- Potential security concerns with work configs

### Multiple Repositories

**Structure:**
```yaml
dotfiles:
  - "~/.dotfiles/personal"    # git@github.com:user/personal-dotfiles.git
  - "~/.dotfiles/work"        # git@company.com:user/work-dotfiles.git
  - "~/.dotfiles/projects"    # git@github.com:user/project-dotfiles.git
```

**Pros:**
- Separate access control
- Independent versioning
- Modular sharing

**Cons:**
- Complex setup and maintenance
- Multiple repositories to manage

### Hybrid Approach

Combine single and multiple repositories strategically:

```yaml
dotfiles:
  - "~/.dotfiles/local"       # Local-only (no git)
  - "~/.dotfiles/personal"    # Personal repository
  - "~/.dotfiles/shared"      # Team repository
```

## Configuration Best Practices

### 1. Use Descriptive Directory Names

```yaml
# Good
dotfiles:
  - "~/.dotfiles/shell-configs"
  - "~/.dotfiles/editor-configs"
  - "~/.dotfiles/development-tools"

# Avoid
dotfiles:
  - "~/.dotfiles/dir1"
  - "~/.dotfiles/misc"
  - "~/.dotfiles/stuff"
```

### 2. Document Your Structure

Create a README in your dotfiles directory:

```markdown
# My Dotfiles Structure

## Directories
- `personal/`: Personal shell and editor configurations
- `work/`: Work-specific git and SSH configurations  
- `system/`: System-level configurations (fonts, themes)

## Priority Order
1. Personal (highest priority)
2. Work (overrides for work context)
3. System (base system configurations)
```

### 3. Use Consistent Naming

Establish and follow naming conventions:

```
# Application-based naming
shell-bash/
shell-zsh/
editor-vim/
editor-vscode/

# Purpose-based naming
personal-shell/
personal-git/
work-ssh/
work-docker/
```

### 4. Handle Secrets Carefully

Keep sensitive configurations in appropriate directories:

```yaml
dotfiles:
  - "~/.dotfiles/public"      # Safe to share publicly
  - "~/.dotfiles/private"     # Personal but not sensitive
  - "~/.dotfiles/secrets"     # Encrypted or local-only
```

## Testing and Validation

### Pre-deployment Testing

Always test configuration changes:

```bash
# Test with dry-run
rotz link --dry-run

# Check for conflicts
rotz link --dry-run | grep -i conflict
```

### Validation Workflow

1. **Backup Current State**
   ```bash
   rotz link --dry-run > current-state.txt
   ```

2. **Apply Changes**
   ```bash
   rotz link
   ```

3. **Verify Applications**
   - Test shell functionality
   - Check editor configurations
   - Verify tool integrations

4. **Document Issues**
   Keep a log of any problems and solutions

### Rollback Procedures

Prepare rollback strategies:

```bash
# Keep backup configurations
cp -r ~/.config ~/.config.backup

# Document working state
rotz link --dry-run > working-state.txt
```

## Performance Optimization

### Minimize Directory Count

While rotz handles multiple directories efficiently, avoid excessive fragmentation:

```yaml
# Good: Logical grouping
dotfiles:
  - "~/.dotfiles/shell"
  - "~/.dotfiles/editors"
  - "~/.dotfiles/tools"

# Avoid: Excessive fragmentation
dotfiles:
  - "~/.dotfiles/bash"
  - "~/.dotfiles/zsh"
  - "~/.dotfiles/vim"
  - "~/.dotfiles/emacs"
  - "~/.dotfiles/git"
  - "~/.dotfiles/tmux"
  # ... 20+ directories
```

### Optimize Glob Patterns

Use specific patterns when possible:

```bash
# More efficient
rotz link /shell/\*

# Less efficient
rotz link /\*\*/\*
```

### Cache Considerations

Consider directory access patterns:
- Frequently changed configurations in early directories
- Stable configurations in later directories
- Network-mounted directories last

## Security Considerations

### Access Control

Structure directories by security requirements:

```yaml
dotfiles:
  - "~/.dotfiles/public"      # World-readable
  - "~/.dotfiles/personal"    # User-only
  - "~/.dotfiles/secure"      # Restricted access
```

### Work Environment Separation

Keep work and personal configurations separate:

```yaml
# Work machine
dotfiles:
  - "~/.dotfiles/work"        # Work configurations first
  - "~/.dotfiles/personal"    # Personal configurations second

# Personal machine  
dotfiles:
  - "~/.dotfiles/personal"    # Personal configurations first
  - "~/.dotfiles/work"        # Work configurations second (if any)
```

## Maintenance and Updates

### Regular Review

Periodically review your directory structure:

1. **Quarterly Review**: Check for unused configurations
2. **Annual Cleanup**: Reorganize if structure becomes unwieldy
3. **Migration Planning**: Plan moves to new organization schemes

### Version Control Best Practices

Tag important milestones:

```bash
git tag -a v1.0 -m "Initial multi-directory setup"
git tag -a v1.1 -m "Added work configurations"
```

### Documentation Updates

Keep documentation current:
- Update README files when structure changes
- Document migration steps
- Maintain troubleshooting guides

## Common Anti-Patterns to Avoid

### 1. Over-Fragmentation

Don't create too many small directories:

```yaml
# Avoid
dotfiles:
  - "~/.dotfiles/bash-aliases"
  - "~/.dotfiles/bash-functions"  
  - "~/.dotfiles/bash-exports"
  - "~/.dotfiles/bash-prompt"

# Better
dotfiles:
  - "~/.dotfiles/shell-bash"
```

### 2. Unclear Hierarchies

Avoid confusing priority orders:

```yaml
# Confusing
dotfiles:
  - "~/.dotfiles/general"
  - "~/.dotfiles/specific"
  - "~/.dotfiles/override"

# Clear
dotfiles:
  - "~/.dotfiles/local-overrides"
  - "~/.dotfiles/personal"
  - "~/.dotfiles/defaults"
```

### 3. Mixed Concerns

Don't mix different organizational strategies:

```yaml
# Inconsistent mixing
dotfiles:
  - "~/.dotfiles/personal"     # Purpose-based
  - "~/.dotfiles/vim"          # Application-based
  - "~/.dotfiles/laptop"       # Environment-based

# Choose one strategy
dotfiles:
  - "~/.dotfiles/personal-shell"
  - "~/.dotfiles/personal-editors"
  - "~/.dotfiles/work-tools"
```

## Migration and Evolution

### Gradual Migration

When restructuring, migrate gradually:

1. **Phase 1**: Create new structure alongside old
2. **Phase 2**: Move configurations incrementally
3. **Phase 3**: Test each migration step
4. **Phase 4**: Remove old structure

### Future-Proofing

Design for evolution:
- Use consistent naming conventions
- Document organizational principles
- Plan for growth and change
- Keep migration paths simple
