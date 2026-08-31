# Troubleshooting

## Common issues and solutions

### `error: failed to read lock file composer.lock: No such file or directory`

You are running LockGuard in a directory without a `composer.lock` file. Either navigate to the project directory or specify the path:

```sh
lockguard --lock /path/to/project/composer.lock
```

### `error: failed to parse lock file: invalid JSON`

The lock file is not valid JSON. This usually means the file is corrupted or was manually edited. Regenerate it:

```sh
composer update --lock
```

Or restore it from version control:

```sh
git checkout composer.lock
```

### `error: duplicate package 'vendor/pkg' with conflicting versions`

The same package appears in both `packages` and `packages-dev` with different versions. This is a lock file inconsistency — Composer itself should not produce this. Regenerate the lock file:

```sh
rm composer.lock
composer install
```

### `error: Packagist API returned status 502`

Packagist's infrastructure returned a server error. LockGuard retries these automatically (up to 2 times with backoff), but if all retries fail, the error surfaces. Wait a few minutes and re-run:

```sh
lockguard
```

If the error persists, check [Packagist status](https://status.packagist.org/) or their [Twitter account](https://twitter.com/packagist) for outage announcements.

### `error: Packagist API returned status 429`

You are being rate-limited. This is unlikely with LockGuard's sequential, low-volume request pattern, but can happen if you run many audits in quick succession. Wait a minute and try again.

### `error: failed to send request: error sending request`

A network error occurred — DNS resolution failure, connection refused, or timeout. Check your internet connection and whether `packagist.org` is reachable:

```sh
curl -I https://packagist.org
```

If you are behind a corporate proxy or firewall, ensure outbound HTTPS to `packagist.org` is allowed.

### `lockguard: command not found`

The binary is not on your `PATH`. If you installed via cargo:

```sh
# Check where cargo installs binaries
ls ~/.cargo/bin/lockguard

# Add cargo bin to PATH (add to ~/.bashrc or ~/.zshrc for persistence)
export PATH="$HOME/.cargo/bin:$PATH"
```

If you installed via the install script or package manager, verify the binary exists:

```sh
which lockguard
ls /usr/local/bin/lockguard
ls /usr/bin/lockguard
```

### No vulnerabilities found but you expected some

Possible causes:

1. **The advisory was filtered by severity.** Try running without `--min-severity`:

   ```sh
   lockguard
   ```

2. **The installed version is outside the affected range.** Check the advisory's `affected_versions` field — your version may already be patched.

3. **Packagist has no advisory for the package.** Not all packages have published advisories. The absence of findings means Packagist has no matching advisory data, not that the package is guaranteed safe.

4. **The package name casing differs.** LockGuard normalizes package names to lowercase before querying, so this should not be an issue. If you suspect it, check the lock file for unusual formatting.

### `warning: no advisory data for 'vendor/pkg' — coverage unknown`

This is an informational message on stderr, not an error. It means Packagist did not return any entry for that package name. This could mean:

- The package has no published advisories (good).
- The package name is not in Packagist's advisory database (neutral — no data either way).

The message is printed so you know the tool queried for the package but got no response. It does not affect the exit code or the report.

### JSON output is not valid JSON

This should not happen — stdout contains exactly one JSON document. If you see invalid JSON:

1. Make sure you are not mixing text and JSON output. Use `--format json` explicitly.
2. Check that stderr is not being captured alongside stdout. Redirect stderr separately:

   ```sh
   lockguard --format json 2>errors.txt >report.json
   ```

3. If the JSON is still invalid, [report a bug](https://github.com/alinaqi2000/lockguard/issues).

### The install script fails with permission errors

The one-liner install script needs `sudo` to write to `/usr/local/bin`. If you cannot use `sudo`, install to a user-writable directory:

```sh
curl -fsSL https://github.com/alinaqi2000/lockguard/releases/latest/download/lockguard-x86_64-unknown-linux-gnu.tar.gz | tar xz -C ~/.local/bin
chmod +x ~/.local/bin/lockguard
```

Make sure `~/.local/bin` is on your `PATH`.

### LockGuard is slow on a large lock file

LockGuard processes batches sequentially. A lock file with 150 packages results in about 10 requests. Each request takes a fraction of a second under normal conditions, so the total time is typically 2-5 seconds.

If it is significantly slower:

- Check your network latency to `packagist.org`.
- Check if retries are happening (look for `retrying batch` messages on stderr).
- Packagist may be under load — try again later.

## Reporting bugs

If you encounter a bug, please [open an issue](https://github.com/alinaqi2000/lockguard/issues) with:

1. The LockGuard version (`lockguard --version`).
2. The command you ran.
3. The full output (stdout and stderr).
4. The exit code (`echo $?`).
5. Your operating system and architecture.
6. The `composer.lock` file or a redacted version that reproduces the issue.

Do not include credentials, private package names, or any data you consider sensitive. Package names from `composer.lock` are sufficient to reproduce most issues.
