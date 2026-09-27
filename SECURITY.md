# Security Policy

## Reporting a Vulnerability

**Do not open public issues for security vulnerabilities.**

Report privately via [GitHub Security Advisories](https://github.com/jarpex/siligpu/security/advisories/new)

## What to Include

- Description of the vulnerability
- Steps to reproduce
- Proof of concept (code, commands, screenshots)
- Affected versions
- Potential impact (DoS, code execution, data leak, etc.)

## Before You Report

We run automated security scanning in CI:

- `cargo audit` — Rust dependency vulnerabilities
- `cargo deny` — license and dependency policy checks
- `trivy` — SBOM vulnerability scanning
- `gitleaks` — secrets detection
- `clippy` — Rust static analysis (SAST)

Check that your finding is not already detected. Also review our [VEX statements](vex/) for known CVEs marked as not applicable.

## Supported Versions

Only the latest release receives security updates.

## Disclosure

We follow coordinated disclosure. Public disclosure happens after a fix is released.
