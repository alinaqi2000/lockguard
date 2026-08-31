# Building from Source

## Prerequisites

- [Rust](https://rustup.rs/) (stable toolchain)
- Git

## Clone and build

```sh
git clone https://github.com/alinaqi2000/lockguard.git
cd lockguard
cargo build --release
```

The binary is at `target/release/lockguard`.

## Quality gates

Before submitting changes, all four gates must pass:

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
cargo build --release
```

## Running tests

Tests use a local mock HTTP server ([wiremock](https://crates.io/crates/wiremock)). No live Packagist requests are made during testing.

```sh
cargo test --all
```

The test suite contains 99 tests:

- 89 unit tests (CLI, lock parsing, Packagist DTOs, HTTP client, version matching, severity, filtering, rendering)
- 10 integration tests (full pipeline with mock server: clean, vulnerable, empty, malformed, missing, conflict, API error, JSON, severity filtering)

## Test fixtures

Fixtures live in `tests/fixtures/`:

```
tests/fixtures/
├── locks/
│   ├── clean.lock
│   ├── vulnerable.lock
│   ├── empty.lock
│   ├── malformed.lock
│   └── duplicate-conflict.lock
└── packagist/
    ├── clean.json
    └── vulnerable.json
```

These are synthetic files with fake package names. No real project data or sensitive information is included.

## Project structure

```
src/
├── main.rs          process boundary: parse CLI, run, exit
├── lib.rs           orchestration: read lock, fetch advisories, audit, render
├── cli.rs           clap argument parsing and value enums
├── error.rs         typed error taxonomy
├── lock.rs          composer.lock DTOs, parsing, normalization
├── packagist.rs     HTTP client, wire DTOs, batching, retry
├── audit.rs         version matching, severity, filtering, result model
└── report.rs        text and JSON rendering
```

## Dependency direction

```
main → cli + lib orchestration
lib → lock + packagist + audit + report
report → audit result model (read-only)
packagist → transport DTOs only
audit → domain types + composer-semver
lock → lock-file DTOs only
```

Rules enforced by module separation:

- `report` does not call Packagist or parse constraints.
- `packagist` does not choose exit codes or render user output.
- `lock` does not silently discard malformed required fields.
- `main` does not contain domain matching logic.
- Transport casing and nullable fields are converted once at the API boundary.

## Dependencies

| Crate | Version | Purpose | License |
|---|---|---|---|
| clap | 4 | CLI argument parsing | MIT/Apache-2.0 |
| serde | 1 | Serialization framework | MIT/Apache-2.0 |
| serde_json | 1 | JSON parsing and output | MIT/Apache-2.0 |
| reqwest | 0.12 | HTTP client (rustls) | MIT/Apache-2.0 |
| tokio | 1 | Async runtime | MIT |
| thiserror | 2 | Error derive macro | MIT/Apache-2.0 |
| composer-semver | 0.2 | Composer version constraint matching | EUPL-1.2 |
| wiremock | 0.6 | Mock HTTP server (dev only) | MIT |

## Releasing

Releases are triggered by pushing a version tag:

```sh
# 1. Bump version in Cargo.toml
# 2. Update Cargo.lock
cargo check

# 3. Commit
git add -A
git commit -m "chore: bump version to 0.2.0"

# 4. Tag and push
git tag v0.2.0
git push origin main v0.2.0
```

The tag push triggers two GitHub Actions workflows:

- **publish.yml** — publishes the crate to crates.io
- **release.yml** — builds Linux binaries (.tar.gz, .deb, .rpm) and attaches them to a GitHub Release

No manual publishing steps are required.
