#!/bin/bash

# VS Code Extension Installation and Test Script
# This script helps you install dependencies and test the extension

set -e

echo "🗑️ Garbage Code Hunter - VS Code Extension Setup"
echo "=================================================="

# Check if Node.js is installed
if ! command -v node &> /dev/null; then
    echo "❌ Node.js is not installed. Please install Node.js first."
    echo "   Visit: https://nodejs.org/"
    exit 1
fi

# Check if the main CLI tool is available
if ! command -v garbage-code-hunter &> /dev/null; then
    echo "⚠️  garbage-code-hunter CLI not found in PATH"
    echo "   Building from source..."
    
    # Build the main project
    cd ..
    cargo build --release
    
    # Add to PATH for this session
    export PATH="$PWD/target/release:$PATH"
    cd vscode-extension
    
    if ! command -v garbage-code-hunter &> /dev/null; then
        echo "❌ Failed to build garbage-code-hunter CLI"
        echo "   Please run 'cargo install --path .' from the project root"
        exit 1
    fi
fi

echo "✅ garbage-code-hunter CLI found: $(which garbage-code-hunter)"

# Install Node.js dependencies
echo ""
echo "📦 Installing Node.js dependencies..."
npm install

# Compile TypeScript
echo ""
echo "🔨 Compiling TypeScript..."
npm run compile

# Test the CLI with JSON output
echo ""
echo "🧪 Testing CLI JSON output..."
garbage-code-hunter test-files/garbage_example.rs --format json --lang en-US > test-output.json

if [ -s test-output.json ]; then
    echo "✅ JSON output test successful!"
    echo "   Found $(jq length test-output.json 2>/dev/null || echo "some") issues in test file"
else
    echo "⚠️  No issues found in test file (this might be expected)"
fi

# Check if VS Code is available
if command -v code &> /dev/null; then
    echo ""
    echo "🚀 VS Code found! You can now test the extension:"
    echo ""
    echo "   1. Open VS Code in this directory:"
    echo "      code ."
    echo ""
    echo "   2. Press F5 to launch Extension Development Host"
    echo ""
    echo "   3. In the new window, open: test-files/garbage_example.rs"
    echo ""
    echo "   4. Save the file to trigger analysis"
    echo ""
    echo "   5. Check the Problems panel for issues"
    echo ""
else
    echo ""
    echo "⚠️  VS Code not found in PATH"
    echo "   Please install VS Code and add it to your PATH"
    echo "   Or manually open this directory in VS Code"
fi

echo ""
echo "📋 Quick Commands:"
echo "   npm run compile  - Compile TypeScript"
echo "   npm run watch    - Watch for changes"
echo "   npm run package  - Create .vsix package"
echo ""
echo "🎭 Ready to roast some code!"

# Clean up
rm -f test-output.json