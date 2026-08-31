# Introduction

LockGuard is a command-line tool that scans PHP project dependencies for known security vulnerabilities. It reads a `composer.lock` file, queries the official [Packagist Security Advisories API](https://packagist.org/api/security-advisories/?packages[]=monolog/monolog), and reports any packages whose installed versions match a published advisory.

## Why use it

If you manage a PHP project — Laravel, Symfony, Drupal, WordPress plugin, or anything else using Composer — your `composer.lock` pins exact versions of sometimes hundreds of packages. Security advisories are published regularly, but checking them manually is tedious and easy to forget. LockGuard automates the check in a single command.

It is designed for two contexts:

- **Local development** — run it before deploying to catch vulnerable dependencies.
- **CI pipelines** — run it on every push or pull request. The JSON output and documented exit codes make it scriptable.

## What it does

1. Reads `composer.lock` (or a path you specify).
2. Combines `packages` and `packages-dev` sections, deduplicates, and sorts.
3. Sends package names to the Packagist Security Advisories API in small batches.
4. Matches each installed version against advisory constraints using Composer-compatible semantics (caret, tilde, wildcards, ranges, OR, hyphens, stability flags).
5. Filters findings by severity threshold.
6. Outputs a text or JSON report.
7. Returns an exit code: `0` for clean, `1` for vulnerabilities found, `2` for errors.

## What it does not do

LockGuard is an audit tool, not a remediation tool. It will not:

- Modify your `composer.lock` or run `composer update`.
- Suggest which version to upgrade to (though the `affected_versions` field tells you the vulnerable range).
- Cache results or run offline.
- Scan containers, repositories, or non-Composer projects.
- Send alerts or integrate with notification systems.

These are deliberate scope boundaries for the initial release. The tool does one thing: tell you what's vulnerable and why.

## Dependencies

LockGuard is a standalone Rust binary. It does not require PHP, Composer, or any runtime service on your machine. The only network call is to `packagist.org` for advisory data.

The project depends on [`composer-semver`](https://crates.io/crates/composer-semver) for version constraint matching. This crate is a faithful port of PHP's Composer semver library, so matching behavior matches what Composer itself would evaluate.
