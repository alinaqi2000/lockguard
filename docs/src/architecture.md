# Architecture

## Design goals

LockGuard is built around six principles that shape every implementation decision:

| Principle | What it means in practice |
|---|---|
| Fast | One reusable HTTP client, bounded batches (15 packages), explicit timeouts (10s connect, 30s request), minimal dependency features. |
| Standalone | No PHP, Composer, database, daemon, or runtime service required. The binary is self-contained. |
| Deterministic | Identical input produces identical output, ordering, counts, and exit code. No randomness, no timestamps in output. |
| CI-friendly | Stable JSON schema, clean stdout, diagnostics on stderr, documented exit codes, no interactive prompts. |
| Actionable | Every finding shows package, version, advisory ID, severity, affected range, and a link to full details. |
| Small scope | Lock parsing, advisory lookup, matching, filtering, reporting. Nothing else. |

## Module layout

```
src/
├── main.rs          process boundary
├── lib.rs           orchestration
├── cli.rs           argument parsing
├── error.rs         error taxonomy
├── lock.rs          lock file parsing
├── packagist.rs     HTTP client + DTOs
├── audit.rs         matching + filtering
└── report.rs        rendering
```

Each module has a single responsibility and a clear boundary with its neighbors.

## Data flow

```
CLI arguments (clap)
  │
  ▼
read composer.lock ──► lock::parse_lock_file
  │                      │
  │                      ▼
  │                   packages + packages-dev
  │                   deduplicate, sort, validate
  │
  ▼
fetch advisories ──► packagist::Client::fetch_advisories
  │                    │
  │                    ├── batch into groups of 15
  │                    ├── sequential HTTP GET requests
  │                    ├── retry transient errors (429/502/503/504)
  │                    └── merge responses
  │
  ▼
match versions ──► audit::run_audit
  │                  │
  │                  ├── for each advisory, test installed version
  │                  │   against affected_versions constraint
  │                  ├── normalize severity
  │                  ├── filter by --min-severity threshold
  │                  └── sort findings
  │
  ▼
render output ──► report::render_text | report::render_json
  │
  ▼
exit code (0 | 1 | 2)
```

## Version matching

The audit engine uses the [`composer-semver`](https://crates.io/crates/composer-semver) crate to evaluate whether an installed version falls within an advisory's `affectedVersions` constraint string.

This crate is a Rust port of PHP's Composer semver library. It handles:

- Caret ranges: `^1.2.3` → `>=1.2.3, <2.0.0`
- Tilde ranges: `~1.2.3` → `>=1.2.3, <1.3.0`
- Wildcards: `1.2.*` → `>=1.2.0, <1.3.0`
- Comparison operators: `>=`, `>`, `<`, `<=`, `=`
- AND (comma or space): `>=1.0.0,<2.0.0`
- OR (double pipe): `>=1.0.0,<2.0.0 || >=3.0.0,<4.0.0`
- Hyphen ranges: `1.0.0 - 2.0.0`
- Stability flags: `@stable`, `@dev`
- Prerelease versions: `1.0.0-beta1`
- `v` prefix: `v1.2.3` is equivalent to `1.2.3`
- Dev branches: `dev-main`, `dev-master`

Using a dedicated crate rather than an ad-hoc parser ensures matching behavior is consistent with Composer itself.

## Batching

Package names are sent to Packagist in batches of 15 per HTTP request. The batch size was chosen after live testing: 50 packages per request produced URLs long enough to trigger HTTP 502 errors from Packagist's infrastructure. 15 packages keeps URLs short while limiting request count.

Batches are processed sequentially — no concurrent requests. Packagist asks clients to stay at or below 10 concurrent requests, and sequential processing is simpler and deterministic.

Package names are sorted alphabetically before batching. This means the same lock file always produces the same request sequence, regardless of how packages are ordered in the lock file.

## Retry behavior

Transient HTTP errors (429, 502, 503, 504) and transport errors (connection reset, timeout) trigger up to 2 retries:

| Attempt | Delay before request |
|---|---|
| 1 (initial) | none |
| 2 (first retry) | 2 seconds |
| 3 (second retry) | 4 seconds |

If all 3 attempts fail, the error is reported and the audit exits with code `2`. Non-transient errors (400, 404, 500, malformed JSON) fail immediately without retry.

Retry progress messages appear on stderr:

```
retrying batch (attempt 2/3) after 2s...
```

## Error taxonomy

Errors are typed at the module boundary and converted once at the API edge:

| Error | Source | Exit code |
|---|---|---|
| Lock file not found | `lock.rs` | 2 |
| Invalid JSON in lock file | `lock.rs` | 2 |
| Missing required fields | `lock.rs` | 2 |
| Conflicting versions | `lock.rs` | 2 |
| HTTP client build failure | `packagist.rs` | 2 |
| HTTP request failure (after retries) | `packagist.rs` | 2 |
| Non-success HTTP status (after retries) | `packagist.rs` | 2 |
| Malformed API response | `packagist.rs` | 2 |
| Report rendering failure | `report.rs` | 2 |

Errors include context in the message — the file path, the URL, or the HTTP status code — so the user can diagnose the problem without reading source code.

## Testing strategy

- **Unit tests** live in each module under `#[cfg(test)] mod tests`. They test individual functions in isolation.
- **Integration tests** live in `tests/integration.rs`. They spin up a mock HTTP server with wiremock and test the full pipeline end-to-end.
- **No live Packagist calls** in the test suite. All HTTP responses are mocked.
- **Fixtures** in `tests/fixtures/` provide synthetic lock files and API responses with fake package names.

The test suite covers:

- CLI argument parsing (defaults, custom paths, invalid values)
- Lock file parsing (valid, invalid, missing, extra fields, duplicates, conflicts)
- Packagist DTO deserialization (all field types, nullables)
- HTTP client (success, batching, dedup, sorting, status errors, malformed JSON, retries)
- Version matching (caret, tilde, wildcard, OR, hyphen, v-prefix, boundaries, multiple advisories)
- Severity normalization and threshold filtering (all combinations)
- Text and JSON rendering (empty, populated, optional fields, null policy, ordering)
- Integration scenarios (clean, vulnerable, empty, malformed, missing, conflict, API error, JSON, severity filter)
