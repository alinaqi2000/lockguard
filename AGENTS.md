# LockGuard

Audit `composer.lock` for known security vulnerabilities using the Packagist
Security Advisories API.

## Session protocol

Before implementing anything, read the Obsidian planning notes:

1. `Obsidian Vault/LockGuard/START-HERE.md`
2. `Obsidian Vault/LockGuard/00-overview/Project Summary.md`
3. `Obsidian Vault/LockGuard/00-overview/Master Workplan.md`

Select the next unblocked task, record it in the session log, and mark exactly
one task as in progress.

## Project principles

- **Fast** — one HTTP client, bounded batches, explicit timeouts.
- **Standalone** — no PHP, Composer, database, or daemon required.
- **Deterministic** — identical input yields identical output and exit code.
- **CI-friendly** — stable JSON, clean stdout, diagnostics on stderr.
- **Actionable** — show package, version, advisory ID, severity, link.
- **Small scope** — lock parsing, advisory lookup, matching, reporting only.

## Verification commands

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
cargo build --release
```

All four must pass before any task is marked complete.

## Out of scope for v0.1.0

No caching, offline mode, alternate sources, config files, SARIF, update
automation, web UI, database, accounts, or hosted service.

## Architecture

See `Obsidian Vault/LockGuard/00-overview/Architecture.md` for the full
architecture, module boundaries, and dependency direction rules.

## Commit rules

- Use conventional commit prefixes: `feat:`, `fix:`, `ci:`, `chore:`, `docs:`, `test:`, `refactor:`
- Examples: `feat: initial commit`, `fix: lock file parsing edge case`, `ci: add release workflow`
- NEVER add AI attribution, "Generated with Devin", or Co-Authored-By lines
- NEVER add any bot or AI as a co-author
- Keep commit messages short and focused on the "why"
