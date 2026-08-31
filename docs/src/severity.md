# Severity Handling

Packagist advisories may include a `severity` field, but it is optional. Some advisories have no severity at all, and some use labels that LockGuard does not recognize. This chapter explains how severity is normalized, displayed, and filtered.

## Severity levels

LockGuard recognizes four severity levels, plus an explicit unknown state:

| Level | String value | Numeric rank |
|---|---|---|
| Critical | `critical` | 4 |
| High | `high` | 3 |
| Medium | `medium` | 2 |
| Low | `low` | 1 |
| Unknown | `unknown` | 1 |

## Normalization

When an advisory includes a `severity` field, LockGuard normalizes it case-insensitively:

- `"high"`, `"HIGH"`, `"High"` → `high`
- `"critical"`, `"CRITICAL"` → `critical`
- `"medium"`, `"Medium"` → `medium`
- `"low"`, `"LOW"` → `low`

Any other value — including unrecognized strings like `"moderate"`, `"info"`, or `"none"` — becomes `unknown`. If the `severity` field is absent or `null`, the finding is also `unknown`.

Severity is **never inferred** from the advisory title, CVE, advisory ID, or source. If Packagist does not provide a severity, the finding is reported as `unknown` — not guessed.

## Threshold filtering

The `--min-severity` flag controls which findings appear in the report. A finding is included if its severity rank is greater than or equal to the threshold rank.

### Truth table

| Finding severity | `--min-severity low` | `--min-severity medium` | `--min-severity high` | `--min-severity critical` |
|---|---|---|---|---|
| Critical | included | included | included | included |
| High | included | included | included | excluded |
| Medium | included | included | excluded | excluded |
| Low | included | excluded | excluded | excluded |
| Unknown | included | excluded | excluded | excluded |

### Unknown severity behavior

Unknown-severity findings are treated as rank 1 (same as low). This means:

- At the default `--min-severity low`, unknown-severity findings are **always reported**. This is the honest default — if we don't know the severity, we report it so the user can investigate.
- At `--min-severity medium` or higher, unknown-severity findings are **excluded**. We cannot claim an unknown-severity advisory meets a higher bar.

This behavior is consistent across text output, JSON output, summary counts, and exit codes. The summary includes a separate `unknown` count so you can see how many findings had no severity.

## Display

### Text format

Severity is shown as an uppercase label in brackets:

```
[CRITICAL] vendor/pkg 1.0.0
[HIGH] vendor/pkg 2.0.0
[MEDIUM] vendor/pkg 3.0.0
[LOW] vendor/pkg 4.0.0
[UNKNOWN] vendor/pkg 5.0.0
```

### JSON format

Severity is a lowercase string:

```json
"severity": "high"
```

```json
"severity": "unknown"
```

## Summary counts

The summary section counts findings by severity, including unknown:

```
Summary:
  - Critical: 0
  - High: 4
  - Medium: 2
  - Low: 0
  - Unknown: 0
```

In JSON:

```json
"summary": {
  "critical": 0,
  "high": 4,
  "medium": 2,
  "low": 0,
  "unknown": 0
}
```

The counts reflect the findings **after** severity filtering. If `--min-severity high` excludes low and medium findings, they do not appear in the summary counts.
