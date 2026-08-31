# Network and Privacy

LockGuard makes network requests to Packagist to fetch security advisory data. This chapter documents exactly what is sent, what is stored, and what is not.

## What is sent

When you run LockGuard, it sends HTTP GET requests to:

```
https://packagist.org/api/security-advisories/?packages[]=vendor/package&packages[]=...
```

The only data transmitted is **package names** from your `composer.lock`. For example:

```
packages[]=monolog/monolog
packages[]=symfony/console
packages[]=laravel/framework
```

Package names are sent in batches of 15 per request, sorted alphabetically for deterministic ordering.

## What is not sent

LockGuard does **not** transmit:

- Your `composer.lock` file contents
- Package versions (only names are sent to query advisories)
- Project name or path
- Your name, email, or any identifying information
- Credentials or authentication tokens
- Environment variables
- File system paths
- Any data about packages not in your lock file

## HTTP behavior

- **Method:** GET
- **Endpoint:** `https://packagist.org/api/security-advisories/`
- **User-Agent:** `lockguard/0.1.0` (identifies the client to Packagist, as their API guidelines request)
- **Authentication:** none — the public endpoint requires no credentials
- **Timeouts:** 10 second connect, 30 second request
- **Retries:** up to 2 retries with exponential backoff (2s, 4s) for transient errors (429, 502, 503, 504)
- **Concurrency:** sequential — one request at a time, no parallel requests
- **TLS:** uses rustls, no system OpenSSL dependency

## What is stored

LockGuard stores nothing on disk. There is no cache, no log file, no config file, no database. Each run is independent and stateless.

## What Packagist sees

Packagist sees that someone at your IP address queried advisory data for specific package names. This is the same information Packagist receives when Composer itself checks for updates. The query pattern is standard API usage.

## Offline usage

LockGuard requires network access to function. It cannot run offline because advisory data must be fetched from Packagist. If the network is unavailable, the tool exits with code `2` and an error message on stderr.

## Privacy summary

| Data | Sent to Packagist | Stored locally |
|---|---|---|
| Package names | yes | no |
| Package versions | no | no |
| Lock file contents | no | no |
| Project path | no | no |
| User identity | no | no |
| Audit results | no | no |
| Credentials | no | no |
