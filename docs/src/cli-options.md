# Command-Line Options

LockGuard accepts the following options. All have sensible defaults, so you can run `lockguard` with no arguments in a directory containing `composer.lock`.

## `--lock <PATH>`

Path to the Composer lock file to audit.

**Default:** `composer.lock`

```sh
lockguard --lock /var/www/myapp/composer.lock
```

The file must be valid JSON with a `packages` array. A `packages-dev` array is optional and defaults to empty. Other fields in the lock file (like `content-hash`, `aliases`, `stability-flags`) are ignored.

## `--format <text|json>`

Output format for the audit report.

**Default:** `text`

```sh
lockguard --format json
```

- `text` — human-readable report with findings and summary.
- `json` — a single JSON document on stdout, suitable for piping to `jq` or another tool.

In both modes, progress messages and warnings go to stderr. stdout contains only the report.

## `--min-severity <low|medium|high|critical>`

Minimum severity threshold. Findings below this severity are excluded from the report.

**Default:** `low` (reports everything)

```sh
lockguard --min-severity high
```

The threshold includes the named severity and above. For example, `--min-severity medium` reports medium, high, and critical findings, but excludes low.

See the [Severity Handling](./severity.md) chapter for how unknown-severity advisories interact with the threshold.

## `--help`

Prints usage information and exits.

```sh
lockguard --help
```

```
Audit composer.lock for known security vulnerabilities

Usage: lockguard [OPTIONS]

Options:
      --lock <LOCK>                  Path to composer.lock [default: composer.lock]
      --format <FORMAT>              Output format [default: text] [possible values: text, json]
      --min-severity <MIN_SEVERITY>  Minimum severity to report [default: low] [possible values: low, medium, high, critical]
  -h, --help                         Print help
  -V, --version                      Print version
```

## `--version`

Prints the binary version and exits.

```sh
lockguard --version
```

```
lockguard 0.1.4
```

## Combining options

All options can be combined:

```sh
lockguard --lock /path/to/composer.lock --format json --min-severity critical
```

This audits a specific lock file, outputs JSON, and only reports critical-severity findings.
