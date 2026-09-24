default:
    @just --list

build-clean:
    ./scripts/build-clean.sh

build:
    cargo build --release

sbom:
    mkdir -p sbom
    cargo cyclonedx --override-filename bom --format json
    mv bom.json sbom/
    ./scripts/clean-sbom.sh sbom/bom.json

secrets:
    rm -f sbom/gitleaks.sarif
    mkdir -p sbom
    gitleaks detect --source . --report-format sarif --report-path sbom/gitleaks.sarif --no-git --log-level warn

audit-gate:
    cargo audit --deny warnings

audit-sarif:
    mkdir -p sbom
    cargo audit --format sarif > sbom/cargo-audit.sarif

sca-rust:
    mkdir -p sbom
    cargo audit --format sarif > sbom/cargo-audit.sarif

sca-general: sbom
    mkdir -p sbom
    trivy sbom sbom/bom.json --scanners vuln,license --format sarif --output sbom/trivy-vuln.sarif --severity HIGH,CRITICAL --disable-telemetry --quiet

scan: audit-sarif audit-gate sca-general secrets

licenses:
    cargo about generate about.hbs -o THIRD_PARTY_LICENSES.html

clean-licenses:
    rm -f THIRD_PARTY_LICENSES.html

check-licenses:
    cargo deny check licenses bans sources

compliance: check-licenses scan licenses