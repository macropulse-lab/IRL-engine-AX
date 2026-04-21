# Contributing to IRL Engine

Thank you for your interest in IRL Engine.

## Before You Open a PR

- For bug fixes and small improvements, open an issue first to confirm the fix is wanted.
- For new features or architectural changes, open an issue and wait for a response before writing code.
- Security vulnerabilities must go through the responsible disclosure process in [SECURITY.md](SECURITY.md) — do not open a public issue.

## Development Setup

```bash
git clone https://github.com/GabrielGauss/IRL-engine-AX.git
cd IRL-engine-AX

cp .env.example .env
# Set MTA_MODE=mock and LAYER2_ENABLED=false for local development

# Start PostgreSQL
docker compose -f docker-compose.standalone.yml up -d postgres

# Run migrations
sqlx migrate run

# Build and run
cargo run
```

Swagger UI is available at `http://localhost:4000/swagger-ui/` once the server is running.

## Code Standards

- Run `cargo fmt` before committing.
- Run `cargo clippy -- -D warnings` and resolve all warnings.
- All new code paths should be covered by tests in `tests/integration.rs`.
- Do not introduce `unwrap()` or `expect()` outside of tests.

## Pull Request Checklist

- [ ] `cargo fmt` applied
- [ ] `cargo clippy -- -D warnings` passes
- [ ] `cargo test` passes
- [ ] New behavior covered by integration tests
- [ ] CHANGELOG.md entry added under `## [Unreleased]`

## Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add heartbeat sequence gap detection
fix: reject authorize when agent is suspended
docs: clarify bind-execution timeout behavior
chore: upgrade sqlx to 0.8
```

## Questions

Open a discussion in the [GitHub Discussions](https://github.com/GabrielGauss/IRL-engine-AX/discussions) tab or email hello@macropulse.live.
