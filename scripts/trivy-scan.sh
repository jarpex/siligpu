#!/bin/sh
set -euo pipefail

SBOM="$1"
OUTPUT="$2"

VEX_FLAG=""
if [ -f vex/siligpu.vex.json ]; then
    VEX_FLAG="--vex vex/siligpu.vex.json"
fi

trivy sbom "$SBOM" --scanners vuln,license --format sarif --output "$OUTPUT" --severity HIGH,CRITICAL --disable-telemetry --quiet $VEX_FLAG
