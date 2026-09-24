#!/bin/sh
set -euo pipefail

SBOM="$1"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

VERSION=$(grep -E '^version\s*=' "$SCRIPT_DIR/../Cargo.toml" | head -n 1 | sed 's/.*"\(.*\)".*/\1/')

if [ -z "$VERSION" ]; then
    echo "ERROR: Could not extract version from Cargo.toml" >&2
    exit 1
fi

jq --arg version "$VERSION" -f "${SCRIPT_DIR}/clean-sbom.jq" "$SBOM" > "${SBOM}.tmp" && mv "${SBOM}.tmp" "$SBOM"