# Output Formats

LockGuard produces two output formats from the same internal audit result. Both contain identical information — the format only affects presentation.

## Text format

The default. Designed to be readable in a terminal without color or special formatting.

### Clean audit (no findings)

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

### Vulnerable audit

```
Auditing composer.lock...
Found 2 packages with known vulnerabilities.

[CRITICAL] vendor/critical-pkg 1.0.0
  - Advisory: PKSA-aaaa-bbbb-cccc
  - CVE: CVE-2024-1234
  - Remote code execution in critical-pkg
  - Affected: >=1.0.0,<1.2.0
  - Link: https://github.com/advisories/GHSA-xxxx
  - Sources: GitHub:GHSA-xxxx, FriendsOfPHP:vendor/critical-pkg/2024.yaml

[HIGH] vendor/high-pkg 3.4.5
  - Advisory: PKSA-dddd-eeee-ffff
  - SQL injection in high-pkg
  - Affected: >=3.0.0,<3.5.0
  - Link: https://github.com/advisories/GHSA-yyyy
  - Sources: GitHub:GHSA-yyyy

Summary:
  - Total packages: 42
  - Vulnerable packages: 2
  - Critical: 1
  - High: 1
  - Medium: 0
  - Low: 0
  - Unknown: 0
```

### Finding fields

Each finding block contains:

| Field | Always shown | Description |
|---|---|---|
| Severity label | yes | `[CRITICAL]`, `[HIGH]`, `[MEDIUM]`, `[LOW]`, or `[UNKNOWN]` |
| Package and version | yes | `vendor/package 1.2.3` |
| Advisory ID | yes | Stable Packagist advisory identifier |
| CVE | only if present | CVE identifier from the advisory |
| Title | yes | Short description of the vulnerability |
| Affected versions | yes | Composer constraint string showing the vulnerable range |
| Link | only if present | URL to the full advisory details |
| Sources | only if present | Comma-separated list of source name and remote ID |

### Ordering

Findings are sorted by:

1. Severity descending (critical first, unknown last)
2. Package name alphabetically
3. Installed version alphabetically
4. Advisory ID alphabetically

This ordering is deterministic — the same input always produces the same output.

## JSON format

Enabled with `--format json`. Produces a single JSON document on stdout.

### Schema

```json
{
  "total_packages": 42,
  "vulnerable_packages": 1,
  "findings": [
    {
      "package": "league/commonmark",
      "version": "2.8.3",
      "advisory_id": "PKSA-1q6p-sqkj-8mmj",
      "severity": "high",
      "cve": null,
      "title": "league/commonmark: Denial of service via duplicate footnote definitions",
      "affected_versions": ">=1.5.0,<2.9.0",
      "link": "https://github.com/advisories/GHSA-jfm3-95jq-q3rf",
      "sources": [
        {
          "name": "GitHub",
          "remote_id": "GHSA-jfm3-95jq-q3rf"
        }
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

### Top-level fields

| Field | Type | Description |
|---|---|---|
| `total_packages` | integer | Total number of packages audited (packages + packages-dev, deduplicated) |
| `vulnerable_packages` | integer | Number of unique packages with at least one reportable finding |
| `findings` | array | One entry per matching advisory, sorted as described above |
| `summary` | object | Count of findings by severity |

### Finding fields

| Field | Type | Nullable | Description |
|---|---|---|---|
| `package` | string | no | Lowercased package name |
| `version` | string | no | Installed version exactly as in the lock file |
| `advisory_id` | string | no | Packagist advisory identifier |
| `severity` | string | no | `critical`, `high`, `medium`, `low`, or `unknown` |
| `cve` | string | yes | CVE identifier, or `null` if not assigned |
| `title` | string | no | Advisory title |
| `affected_versions` | string | no | Composer constraint string for the vulnerable range |
| `link` | string | yes | URL to advisory details, or `null` if not provided |
| `sources` | array | no | Array of source objects (empty array if none) |

### Source object

| Field | Type | Description |
|---|---|---|
| `name` | string | Source name (e.g., `GitHub`, `FriendsOfPHP/security-advisories`) |
| `remote_id` | string | Source-specific identifier (e.g., GHSA ID) |

### Null policy

Nullable fields (`cve`, `link`) always serialize as `null` when absent — they are never omitted. The `sources` array is always present, empty if no sources exist. This is consistent across all responses and versions.

### Parsing with jq

Count findings:

```sh
lockguard --format json | jq '.findings | length'
```

List affected packages:

```sh
lockguard --format json | jq -r '.findings[].package' | sort -u
```

Extract CVEs:

```sh
lockguard --format json | jq '.findings[].cve' | grep -v null
```

Check if any critical findings exist:

```sh
lockguard --format json | jq '.summary.critical > 0'
```

## Stream separation

In both formats:

- **stdout** — the report only (text or JSON).
- **stderr** — progress messages (`Auditing composer.lock...`), warnings (`warning: no advisory data for 'vendor/pkg' — coverage unknown`), retry notices, and error messages.

This means `lockguard --format json > report.json` produces a clean JSON file with no interleaved text. Errors still appear in the terminal because stderr is not redirected.
