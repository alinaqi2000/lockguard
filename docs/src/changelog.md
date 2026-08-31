# Changelog

All notable changes to LockGuard are documented here. Versions follow [Semantic Versioning](https://semver.org/).

## v0.1.4 — 2026-08-31

### Fixed

- Reduced batch size from 50 to 15 packages per request. The previous batch size produced URLs that triggered HTTP 502 errors from Packagist's infrastructure on larger lock files.
- Added retry with exponential backoff (2 retries, 2s/4s delays) for transient HTTP errors (429, 502, 503, 504). Non-transient errors fail immediately.
- Fixed install script permission errors by using a temporary directory and `install -m 755` instead of extracting directly to `/tmp` and using `mv` + `chmod`.

## v0.1.3 — 2026-08-31

### Fixed

- Fixed the one-liner install script not being included as a release asset, causing a 404 when users ran `curl -fsSL .../install.sh | sh`.

## v0.1.2 — 2026-08-31

### Fixed

- Added `packaging/install.sh` to release assets so the one-liner install works.

## v0.1.1 — 2026-08-29

### Fixed

- Corrected repository URLs in `Cargo.toml` and `README.md` from `github.com/alinaqi` to `github.com/alinaqi2000`.

## v0.1.0 — 2026-08-29

### Added

- Initial release.
- Audit `composer.lock` files against the Packagist Security Advisories API.
- Composer-compatible version constraint matching via the `composer-semver` crate.
- Text and JSON output formats with stable schema.
- Severity normalization (critical, high, medium, low, unknown) and threshold filtering.
- Documented exit codes: 0 (clean), 1 (vulnerabilities found), 2 (operational error).
- Both `packages` and `packages-dev` sections audited with deduplication.
- Bounded sequential batching (15 packages per request).
- Explicit connect (10s) and request (30s) timeouts.
- 99 tests (89 unit + 10 integration) with mocked HTTP server.
- CI workflow: fmt, clippy, test, release build.
- crates.io publication workflow triggered by version tags.
- Linux binary release workflow: x86_64 and aarch64 tar.gz, .deb, .rpm with SHA256 checksums.
- Install script for systems without a package manager.
- AUR PKGBUILD for Arch Linux.
