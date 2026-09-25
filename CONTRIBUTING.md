# Contributing

Thanks for your interest! This is a solo-maintained project.

## Requirements

Install [just](https://github.com/casey/just) as your command runner:

```bash
brew install just
```

Install Rust development tools:

```bash
cargo install cargo-audit cargo-deny cargo-about cargo-fuzz cargo-cyclonedx
rustup component add clippy
```

Install external scanners:

```bash
brew install gitleaks trivy vexctl
```

## Workflow

Run `just` to list all commands:

```bash
cargo test              # all tests (unit, integration, proptest)
just scan               # security gate: fmt, clippy, audit, trivy, gitleaks
just test-props         # property-based tests only
just fuzz-parse         # fuzz parser for 60s (or `just fuzz-parse time=300`)
just compliance         # scan + license/ban checks + regenerate THIRD_PARTY_LICENSES.html
```

## Before Opening a PR

Open an issue for large changes first. For the PR to be accepted, CI runs the same gates locally — please verify beforehand:

1. `cargo test` passes
2. `just scan` passes
3. Run `just fuzz-parse` if you changed `parse_duration`

## Commit Style

We follow [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/):

- Imperative mood (`fix:` not `fixed:`)
- Conventional prefixes: `feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `chore:`, `ci:`
- Use `!` for breaking changes (e.g., `feat!: new API`)

## Security Issues

See [Security Policy](https://github.com/jarpex/siligpu/security).
