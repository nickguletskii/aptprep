#!/bin/bash

# Script to format all files in the aptprep project
# This script formats Rust code, YAML files, JSON files, and Bazel files
#
# Usage: ./scripts/format.sh [OPTIONS]
# Options:
#   --skip-rust        Skip Rust formatting (cargo fmt and clippy)
#   --skip-prettier    Skip prettier formatting (YAML, JSON)
#   --skip-bazel       Skip Bazel formatting (buildifier)
#   --skip-clippy      Skip clippy linting (still runs cargo fmt if --skip-rust not set)
#   --help             Show this help message

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Default flags
SKIP_RUST=false
SKIP_PRETTIER=false
SKIP_BAZEL=false
SKIP_CLIPPY=false

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --skip-rust)
            SKIP_RUST=true
            shift
            ;;
        --skip-prettier)
            SKIP_PRETTIER=true
            shift
            ;;
        --skip-bazel)
            SKIP_BAZEL=true
            shift
            ;;
        --skip-clippy)
            SKIP_CLIPPY=true
            shift
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo "Options:"
            echo "  --skip-rust        Skip Rust formatting (cargo fmt and clippy)"
            echo "  --skip-prettier    Skip prettier formatting (YAML, JSON)"
            echo "  --skip-bazel       Skip Bazel formatting (buildifier)"
            echo "  --skip-clippy      Skip clippy linting (still runs cargo fmt if --skip-rust not set)"
            echo "  --help             Show this help message"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Get the project root directory
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_ROOT"

echo -e "${GREEN}Formatting files in aptprep project...${NC}"

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Format Rust code
if [ "$SKIP_RUST" = false ]; then
    echo -e "${YELLOW}Formatting Rust code...${NC}"
    if command_exists cargo; then
        cargo fmt --all
        echo -e "${GREEN}✓ Rust code formatted${NC}"
    else
        echo -e "${RED}✗ cargo not found, skipping Rust formatting${NC}"
    fi
else
    echo -e "${YELLOW}Skipping Rust formatting${NC}"
fi

# Format YAML and JSON files with prettier if available
if [ "$SKIP_PRETTIER" = false ]; then
    echo -e "${YELLOW}Formatting YAML and JSON files...${NC}"
    if command_exists pnpx && [ -f package.json ]; then
        pnpx prettier --write "**/*.{yml,yaml,json}"
        echo -e "${GREEN}✓ YAML and JSON files formatted${NC}"
    elif command_exists npx; then
        npx prettier --write "**/*.{yml,yaml,json}" 2>/dev/null || {
            echo -e "${YELLOW}⚠ prettier not available, skipping YAML/JSON formatting${NC}"
        }
    else
        echo -e "${YELLOW}⚠ prettier not available, skipping YAML/JSON formatting${NC}"
    fi
else
    echo -e "${YELLOW}Skipping prettier formatting${NC}"
fi

# Format Bazel files
if [ "$SKIP_BAZEL" = false ]; then
    echo -e "${YELLOW}Formatting Bazel files...${NC}"
    if command_exists buildifier; then
        find . -name "*.bzl" -o -name "BUILD*" -o -name "MODULE.bazel" | xargs buildifier
        echo -e "${GREEN}✓ Bazel files formatted${NC}"
    else
        echo -e "${YELLOW}⚠ buildifier not found, skipping Bazel formatting${NC}"
        echo -e "${YELLOW}  Install with: wget -O buildifier https://github.com/bazelbuild/buildtools/releases/download/v6.4.0/buildifier-linux-amd64 && chmod +x buildifier && sudo mv buildifier /usr/local/bin/${NC}"
    fi
else
    echo -e "${YELLOW}Skipping Bazel formatting${NC}"
fi

# Run clippy for additional linting
if [ "$SKIP_CLIPPY" = false ] && [ "$SKIP_RUST" = false ]; then
    echo -e "${YELLOW}Running clippy for additional linting...${NC}"
    if command_exists cargo; then
        cargo clippy --all-targets --all-features -- -D warnings || {
            echo -e "${YELLOW}⚠ Clippy found issues, please review${NC}"
        }
        echo -e "${GREEN}✓ Clippy check completed${NC}"
    else
        echo -e "${RED}✗ cargo not found, skipping clippy${NC}"
    fi
elif [ "$SKIP_CLIPPY" = true ]; then
    echo -e "${YELLOW}Skipping clippy${NC}"
elif [ "$SKIP_RUST" = true ]; then
    echo -e "${YELLOW}Skipping clippy (Rust formatting disabled)${NC}"
fi

echo -e "${GREEN}All formatting completed!${NC}"

# Optional: Check for any uncommitted changes
if command_exists git && git rev-parse --git-dir > /dev/null 2>&1; then
    if ! git diff --quiet; then
        echo -e "${YELLOW}Note: There are uncommitted changes after formatting${NC}"
        echo "Run 'git diff' to see what was changed"
    else
        echo -e "${GREEN}No changes made during formatting${NC}"
    fi
fi