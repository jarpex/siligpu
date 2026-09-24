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


sca-general: sbom
    mkdir -p sbom
    ./scripts/trivy-scan.sh sbom/bom.json sbom/trivy-vuln.sarif

sast:
    mkdir -p sbom
    ./scripts/clippy-sarif.sh sbom/clippy.sarif

scan: fmt-check audit-sarif audit-gate sca-general secrets sast

fmt:
    cargo fmt

fmt-check:
    cargo fmt --check


licenses:
    cargo about generate about.hbs -o THIRD_PARTY_LICENSES.html

clean-licenses:
    rm -f THIRD_PARTY_LICENSES.html

check-licenses:
    cargo deny check licenses bans sources

compliance: check-licenses scan licenses

vex-create vuln subcomponent justification statement:
    mkdir -p vex/statements
    vexctl create \
      --product "pkg:cargo/siligpu@1.0.0" \
      --subcomponents "pkg:cargo/{{subcomponent}}" \
      --vuln "{{vuln}}" \
      --status "not_affected" \
      --justification "{{justification}}" \
      --impact-statement "{{statement}}" \
      --file "vex/statements/siligpu-{{vuln}}.vex.json"
    @just vex-merge

vex-merge:
    vexctl merge vex/statements/*.vex.json > vex/siligpu.vex.json

vex-list:
    @ls -1 vex/statements/*.vex.json 2>/dev/null || echo "No VEX documents found"

test-props:
    cargo test --lib proptests -- --nocapture

fuzz-parse time="60":
    cargo fuzz run fuzz_parse_duration -- -max_total_time={{time}}

fuzz-build:
    cargo fuzz build
