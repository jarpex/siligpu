#!/bin/sh
set -euo pipefail

SBOM="$1"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

jq -f "${SCRIPT_DIR}/clean-sbom.jq" "$SBOM" > "${SBOM}.tmp" && mv "${SBOM}.tmp" "$SBOM"