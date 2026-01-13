#!/bin/bash

# fix-rotz-config.sh
# Script to fix rotz configuration issues that cause "Encountered multiple errors"

set -e

echo "🔧 Fixing rotz configuration issues..."

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

DOTFILES_DIR="/var/home/phreed/.dotfiles"
ROTZ_CONFIG_DIR="/var/home/phreed/.config/rotz"
ROTZ_CONFIG_FILE="$ROTZ_CONFIG_DIR/config.yaml"

# Create rotz config directory if it doesn't exist
if [ ! -d "$ROTZ_CONFIG_DIR" ]; then
    echo -e "${YELLOW}Creating rotz config directory...${NC}"
    mkdir -p "$ROTZ_CONFIG_DIR"
fi

# Backup existing config if it exists
if [ -f "$ROTZ_CONFIG_FILE" ]; then
    echo -e "${YELLOW}Backing up existing config...${NC}"
    cp "$ROTZ_CONFIG_FILE" "$ROTZ_CONFIG_FILE.backup.$(date +%s)"
fi

# Fix 1: Create proper rotz config file
echo -e "${GREEN}✓ Creating proper rotz config file...${NC}"
cat > "$ROTZ_CONFIG_FILE" << 'EOF'
# Rotz configuration file
# This contains global settings for rotz

dotfiles: /var/home/phreed/.dotfiles
link_type: symbolic
shell_command: "bash -c {{ quote \"\" cmd }}"

variables:
  user: "{{ user }}"
  home: "{{ home }}"
EOF

# Fix 2: Fix the problematic dev/dot.yaml file
echo -e "${GREEN}✓ Fixing dev/dot.yaml file...${NC}"
DEV_DOT_FILE="$DOTFILES_DIR/dev/dot.yaml"
if [ -f "$DEV_DOT_FILE" ]; then
    # Backup the original
    cp "$DEV_DOT_FILE" "$DEV_DOT_FILE.backup.$(date +%s)"

    # Create corrected version
    cat > "$DEV_DOT_FILE" << 'EOF'
# Development environment configuration
# This file should only contain dot-level properties (links, installs, depends)
# Config-level properties like dotfiles, link_type, etc. belong in ~/.config/rotz/config.yaml

global:
  installs: false
  depends: []

linux:
  links: {}
  installs: false
  depends: []

darwin:
  links: {}
  installs: false
  depends: []

windows:
  links: {}
  installs: false
  depends: []
EOF
fi

# Fix 3: Fix all empty flatpak dot.yaml files
echo -e "${GREEN}✓ Fixing empty flatpak dot.yaml files...${NC}"
empty_files=0
while IFS= read -r -d '' file; do
    if [ -s "$file" ]; then
        continue  # File is not empty, skip
    fi

    # File is empty, add basic content
    cat > "$file" << 'EOF'
# Flatpak application configuration
# Currently no specific configuration needed

global:
  installs: false
  depends: []
  links: {}
EOF
    empty_files=$((empty_files + 1))
done < <(find "$DOTFILES_DIR/flatpak" -name "dot.yaml" -print0 2>/dev/null || true)

if [ $empty_files -gt 0 ]; then
    echo -e "${GREEN}   Fixed $empty_files empty flatpak dot.yaml files${NC}"
fi

# Fix 4: Check for other potential issues
echo -e "${GREEN}✓ Checking for other potential issues...${NC}"

# Check for any files with invalid YAML syntax
echo -e "${YELLOW}Validating YAML syntax in dot.yaml files...${NC}"
yaml_errors=0
while IFS= read -r -d '' file; do
    # Simple YAML validation using python (most systems have it)
    if command -v python3 >/dev/null 2>&1; then
        if ! python3 -c "
import yaml
import sys
try:
    with open('$file', 'r') as f:
        yaml.safe_load(f)
except yaml.YAMLError as e:
    print('YAML Error in $file: ' + str(e))
    sys.exit(1)
except Exception as e:
    print('Error reading $file: ' + str(e))
    sys.exit(1)
" 2>/dev/null; then
            echo -e "${RED}   ⚠ YAML syntax error in: $file${NC}"
            yaml_errors=$((yaml_errors + 1))
        fi
    fi
done < <(find "$DOTFILES_DIR" -name "dot.yaml" -print0 2>/dev/null || true)

if [ $yaml_errors -eq 0 ]; then
    echo -e "${GREEN}   All YAML files appear to be valid${NC}"
fi

# Fix 5: Set proper permissions
echo -e "${GREEN}✓ Setting proper permissions...${NC}"
chmod 644 "$ROTZ_CONFIG_FILE"
find "$DOTFILES_DIR" -name "dot.yaml" -exec chmod 644 {} \; 2>/dev/null || true

# Test the configuration
echo -e "${GREEN}✓ Testing rotz configuration...${NC}"
cd "$(dirname "$0")"
if cargo run -- --dry-run link >/dev/null 2>&1; then
    echo -e "${GREEN}🎉 Configuration test passed!${NC}"
else
    echo -e "${RED}❌ Configuration test failed. Manual intervention may be needed.${NC}"
    echo -e "${YELLOW}Try running: cargo run -- --dry-run link${NC}"
    echo -e "${YELLOW}to see detailed error messages.${NC}"
fi

echo ""
echo -e "${GREEN}🎉 Rotz configuration fixes completed!${NC}"
echo ""
echo "Summary of changes:"
echo "1. Created proper rotz config file at: $ROTZ_CONFIG_FILE"
echo "2. Fixed dev/dot.yaml to contain only dot-level properties"
echo "3. Fixed $empty_files empty flatpak dot.yaml files"
echo "4. Set proper file permissions"
echo ""
echo "You can now try running: rotz link"
echo ""
if [ $yaml_errors -gt 0 ]; then
    echo -e "${YELLOW}Note: Found $yaml_errors YAML syntax errors that may need manual fixing.${NC}"
fi
