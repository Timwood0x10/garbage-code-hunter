#!/bin/bash
# Install Git hooks for this project
# This script creates symlinks from .git/hooks to the hooks/ directory

set -e

echo "🔗 Installing Git hooks..."

# Get the directory where this script is located (resolve symlinks)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"
HOOKS_DIR="$REPO_ROOT/hooks"
GIT_HOOKS_DIR="$REPO_ROOT/.git/hooks"

echo "   Repo root: $REPO_ROOT"
echo "   Hooks dir: $HOOKS_DIR"
echo "   Git hooks: $GIT_HOOKS_DIR"

# Check if hooks directory exists
if [ ! -d "$HOOKS_DIR" ]; then
    echo "❌ Hooks directory not found: $HOOKS_DIR"
    exit 1
fi

# Check if .git/hooks directory exists
if [ ! -d "$GIT_HOOKS_DIR" ]; then
    echo "❌ .git/hooks directory not found"
    exit 1
fi

# Install each hook
for hook in "$HOOKS_DIR"/*; do
    hook_name=$(basename "$hook")

    if [ "$hook_name" = "install-hooks.sh" ]; then
        continue
    fi

    if [ -f "$hook" ] && [ -x "$hook" ]; then
        ln -sf "../../hooks/$hook_name" "$GIT_HOOKS_DIR/$hook_name"
        echo "✅ Installed: $hook_name"
    elif [ -f "$hook" ]; then
        # Make it executable if it's not already
        chmod +x "$hook"
        ln -sf "../../hooks/$hook_name" "$GIT_HOOKS_DIR/$hook_name"
        echo "✅ Installed: $hook_name (made executable)"
    fi
done

echo ""
echo "✅ Git hooks installed successfully!"
echo "   Now every commit and push will run 'make ci' automatically."
