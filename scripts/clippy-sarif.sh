#!/bin/sh
set -euo pipefail

OUTPUT="${1:-sbom/clippy.sarif}"
mkdir -p "$(dirname "$OUTPUT")"
TMP_FILE=$(mktemp)

trap 'rm -f "$TMP_FILE"' EXIT

echo "[sast] Running clippy with strict rules (-D warnings)..."

set +e

cargo clippy \
    --all-targets \
    --all-features \
    --message-format=json-render-diagnostics \
    -- -D warnings > "$TMP_FILE"

CLIPPY_EXIT=$?
set -e

echo "[sast] Converting to SARIF..."
if command -v clippy-sarif >/dev/null 2>&1; then
    clippy-sarif < "$TMP_FILE" > "$OUTPUT"
else
    echo "[sast] ERROR: clippy-sarif not installed" >&2
    exit 1
fi

# Dedup
if command -v jq >/dev/null 2>&1; then
    jq '.runs[0].results |= unique_by(.ruleId, .locations[0].physicalLocation.artifactLocation.uri, .locations[0].physicalLocation.region.startLine, .locations[0].physicalLocation.region.startColumn)' "$OUTPUT" > "$OUTPUT.tmp" && mv "$OUTPUT.tmp" "$OUTPUT"
fi

FINDINGS=$(jq '[.runs[0].results[]] | length' "$OUTPUT" 2>/dev/null || echo "0")
echo "[sast] SARIF written to $OUTPUT"
echo "[sast] Clippy findings: $FINDINGS"

if [ "$CLIPPY_EXIT" -ne 0 ]; then
    echo ""
    echo "[sast] ❌ ERROR: clippy failed with exit code $CLIPPY_EXIT" >&2
    echo "[sast] Fix all warnings before proceeding." >&2
    exit "$CLIPPY_EXIT"
fi

echo "[sast] ✅ Clippy gate passed"
