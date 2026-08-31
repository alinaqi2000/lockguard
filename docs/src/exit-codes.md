# Exit Codes

LockGuard uses three exit codes. This separation lets CI pipelines distinguish between "no vulnerabilities," "vulnerabilities found," and "the tool itself failed."

| Code | Meaning | When |
|---|---|---|
| `0` | Clean audit | Audit completed successfully and no findings met the severity threshold |
| `1` | Vulnerabilities found | Audit completed successfully and one or more findings met the threshold |
| `2` | Operational error | The audit could not complete due to invalid input, network failure, or malformed data |

## Exit code 0 — clean

The lock file was read, packages were queried, and no reportable vulnerabilities were found. This includes:

- All packages returned with empty advisory arrays (known clean).
- Packages had advisories but none matched the installed version.
- All matching advisories were filtered out by `--min-severity`.

## Exit code 1 — vulnerabilities found

At least one advisory matched an installed version and met the severity threshold. The findings are in the report on stdout.

This is **not** an error. The tool worked correctly — it found vulnerabilities, which is its job. CI pipelines that treat any non-zero exit as a failure should special-case exit code `1`.

## Exit code 2 — operational error

The audit could not complete. Common causes:

- **Lock file missing or unreadable** — the path does not exist or permissions prevent reading.
- **Invalid JSON** — the lock file is not valid JSON.
- **Missing required fields** — the lock file lacks a `packages` array or a package is missing its name or version.
- **Conflicting versions** — the same package appears in `packages` and `packages-dev` with different versions.
- **HTTP error** — Packagist returned a non-transient error (e.g., 404, 400) after retries.
- **Malformed response** — Packagist returned data that could not be parsed as JSON.
- **Network failure** — connection timed out or was refused after retries.

The error message is printed to stderr with enough context to diagnose the problem:

```
error: failed to read lock file /path/to/composer.lock: No such file or directory
```

```
error: Packagist API returned status 500 for https://packagist.org/api/security-advisories?packages[]=vendor/pkg
```

## CI usage examples

### Fail the build on any vulnerability

```sh
lockguard --format json --min-severity high
# exit 0 = clean, exit 1 = fail the build, exit 2 = infrastructure error
```

### Don't fail, just report

```sh
lockguard || true
```

### Distinguish vulnerability from error

```sh
lockguard --format json > report.json
code=$?
if [ $code -eq 1 ]; then
    echo "Vulnerabilities found — see report.json"
    # notify team, create ticket, etc.
elif [ $code -eq 2 ]; then
    echo "Audit failed — check infrastructure"
    exit 1
fi
```

### GitHub Actions

```yaml
- name: Audit dependencies
  run: lockguard --format json --min-severity high
  continue-on-error: true
  id: audit

- name: Report vulnerabilities
  if: steps.audit.outcome == 'failure'
  run: |
    echo "::warning::Vulnerabilities found in composer.lock"
    cat report.json | jq '.findings[]'
```
