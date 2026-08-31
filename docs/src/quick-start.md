# Quick Start

## Basic audit

Navigate to any directory containing a `composer.lock` file and run:

```sh
lockguard
```

Output:

```
Auditing composer.lock...
Found 1 packages with known vulnerabilities.

[HIGH] league/commonmark 2.8.3
  - Advisory: PKSA-1q6p-sqkj-8mmj
  - league/commonmark: Denial of service via duplicate footnote definitions
  - Affected: >=1.5.0,<2.9.0
  - Link: https://github.com/advisories/GHSA-jfm3-95jq-q3rf
  - Sources: GitHub:GHSA-jfm3-95jq-q3rf

Summary:
  - Total packages: 137
  - Vulnerable packages: 1
  - Critical: 0
  - High: 1
  - Medium: 0
  - Low: 0
  - Unknown: 0
```

The exit code is `1` because vulnerabilities were found. You can check it with:

```sh
echo $?
```

## Clean audit

If no vulnerabilities are found:

```
Auditing composer.lock...
No known vulnerabilities found.

Summary:
  - Total packages: 42
  - Vulnerable packages: 0
  - Critical: 0
  - High: 0
  - Medium: 0
  - Low: 0
  - Unknown: 0
```

Exit code is `0`.

## JSON output for scripting

```sh
lockguard --format json
```

This produces a single JSON document on stdout. Diagnostics and progress messages go to stderr, so piping the JSON to another tool works cleanly:

```sh
lockguard --format json | jq '.findings | length'
```

## Filter by severity

Only report high and critical vulnerabilities:

```sh
lockguard --min-severity high
```

## Audit a specific lock file

```sh
lockguard --lock /path/to/project/composer.lock
```

## What to do with findings

LockGuard tells you what's vulnerable and links to the advisory. To fix a finding:

1. Read the advisory link to understand the vulnerability.
2. Check the `affected_versions` field — it tells you the vulnerable range.
3. Update the package to a version outside that range:

```sh
composer update vendor/package
```

4. Re-run LockGuard to confirm the fix:

```sh
lockguard
```
