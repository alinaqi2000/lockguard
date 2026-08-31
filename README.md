# LockGuard

A fast, standalone CLI that audits `composer.lock` files for known security
vulnerabilities using the official [Packagist Security Advisories API](https://packagist.org/api/security-advisories/).

No PHP or Composer installation required.

**Full documentation: https://alinaqi2000.github.io/lockguard/**

## Installation

### Cargo (any OS)

```sh
cargo install lockguard
```

### One-liner (Linux)

```sh
curl -fsSL https://github.com/alinaqi2000/lockguard/releases/latest/download/install.sh | sh
```

### Debian/Ubuntu

```sh
wget https://github.com/alinaqi2000/lockguard/releases/latest/download/lockguard_0.1.4_amd64.deb
sudo dpkg -i lockguard_0.1.4_amd64.deb
```

### Fedora/RHEL/openSUSE

```sh
sudo rpm -i https://github.com/alinaqi2000/lockguard/releases/latest/download/lockguard-0.1.4-1.x86_64.rpm
```

### Build from source

```sh
git clone https://github.com/alinaqi2000/lockguard.git
cd lockguard
cargo build --release
# Binary at target/release/lockguard
```

## Usage

```sh
lockguard [OPTIONS]
```

Run inside a directory containing `composer.lock`:

```sh
$ lockguard
Auditing composer.lock...
Found 1 packages with known vulnerabilities.

[HIGH] monolog/monolog 1.10.0
  - Advisory: PKSA-dmw8-jd8k-q3c6
  - CVE: CVE-2024-1234
  - Header injection in NativeMailerHandler
  - Affected: >=1.8.0,<1.12.0
  - Link: https://github.com/Seldaek/monolog/pull/448
  - Sources: GitHub:GHSA-f57v-q966-7fh6

Summary:
  - Total packages: 42
  - Vulnerable packages: 1
  - Critical: 0
  - High: 1
  - Medium: 0
  - Low: 0
  - Unknown: 0
```

## Options

| Option | Default | Description |
|---|---|---|
| `--lock <PATH>` | `composer.lock` | Path to the lock file |
| `--format <text\|json>` | `text` | Output format |
| `--min-severity <low\|medium\|high\|critical>` | `low` | Minimum severity to report |
| `--help` | | Show help |
| `--version` | | Show version |

## JSON output

```sh
lockguard --format json
```

```json
{
  "total_packages": 42,
  "vulnerable_packages": 1,
  "findings": [
    {
      "package": "monolog/monolog",
      "version": "1.10.0",
      "advisory_id": "PKSA-dmw8-jd8k-q3c6",
      "severity": "high",
      "cve": null,
      "title": "Header injection in NativeMailerHandler",
      "affected_versions": ">=1.8.0,<1.12.0",
      "link": "https://github.com/Seldaek/monolog/pull/448",
      "sources": [
        { "name": "GitHub", "remote_id": "GHSA-f57v-q966-7fh6" }
      ]
    }
  ],
  "summary": {
    "critical": 0,
    "high": 1,
    "medium": 0,
    "low": 0,
    "unknown": 0
  }
}
```

## Exit codes

| Code | Meaning |
|---|---|
| `0` | Audit completed, no reportable vulnerabilities |
| `1` | Audit completed, one or more vulnerabilities found |
| `2` | Operational error (invalid input, network failure, malformed data) |

## Severity handling

Packagist advisories may include a `severity` field. When severity is absent or
unrecognized, it is reported as `unknown`. Unknown-severity advisories are
included at the default `low` threshold and excluded when a higher threshold is
specified. Severity is never inferred from titles, CVEs, or advisory IDs.

## CI usage

```sh
lockguard --format json --min-severity high
```

- Report output goes to stdout; diagnostics go to stderr.
- JSON mode produces exactly one JSON document on stdout.
- Exit code `1` indicates vulnerabilities, not failure.

## Network and privacy

LockGuard sends package names to `packagist.org` to check for advisories. No
credentials, lock file contents, or personal data are transmitted beyond the
package names required for the API query.

## Development

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
cargo build --release
```

Tests use a local mock server — no live Packagist dependency in CI.

## License

MIT

This project depends on [`composer-semver`](https://crates.io/crates/composer-semver)
(EUPL-1.2) for Composer-compatible version constraint matching.
