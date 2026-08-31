# CI/CD Pipelines

LockGuard is designed to run in automated pipelines. The JSON output, documented exit codes, and clean stream separation make it straightforward to integrate.

## GitHub Actions

### Basic vulnerability check

Run LockGuard on every push and fail if high or critical vulnerabilities are found:

```yaml
name: Security audit

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install lockguard
        run: |
          curl -fsSL https://github.com/alinaqi2000/lockguard/releases/latest/download/install.sh | sh

      - name: Audit dependencies
        run: lockguard --min-severity high
```

This fails the build (exit code 1) if any high or critical vulnerabilities are found. Exit code 2 (infrastructure error) also fails the build.

### Report without failing

If you want to see vulnerabilities but not block the pipeline:

```yaml
- name: Audit dependencies
  run: lockguard --format json > audit-report.json || true

- name: Upload report
  uses: actions/upload-artifact@v4
  with:
    name: security-audit
    path: audit-report.json
```

### Separate vulnerability from error

```yaml
- name: Audit dependencies
  id: audit
  run: |
    lockguard --format json --min-severity high > audit-report.json
    echo "exit_code=$?" >> "$GITHUB_OUTPUT"

- name: Handle vulnerabilities
  if: steps.audit.outputs.exit_code == '1'
  run: |
    echo "::warning::Vulnerabilities found"
    jq '.findings[] | {package, severity, advisory_id, link}' audit-report.json

- name: Handle errors
  if: steps.audit.outputs.exit_code == '2'
  run: |
    echo "::error::Security audit failed — check infrastructure"
    exit 1
```

### Scheduled daily check

Run a daily audit to catch newly published advisories:

```yaml
name: Daily security audit

on:
  schedule:
    - cron: '0 8 * * *'

jobs:
  audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install lockguard
        run: curl -fsSL https://github.com/alinaqi2000/lockguard/releases/latest/download/install.sh | sh

      - name: Audit
        run: lockguard --format json --min-severity medium
```

## GitLab CI

```yaml
security_audit:
  stage: test
  image: rust:latest
  before_script:
    - cargo install lockguard
  script:
    - lockguard --format json --min-severity high > audit-report.json
  artifacts:
    paths:
      - audit-report.json
    when: always
  rules:
    - if: $CI_PIPELINE_SOURCE == "merge_request_event"
```

## Jenkins

```groovy
pipeline {
    agent any
    stages {
        stage('Security Audit') {
            steps {
                sh '''
                    curl -fsSL https://github.com/alinaqi2000/lockguard/releases/latest/download/install.sh | sh
                    lockguard --format json --min-severity high > audit-report.json
                '''
            }
            post {
                always {
                    archiveArtifacts artifacts: 'audit-report.json'
                }
                failure {
                    echo 'Security audit found vulnerabilities or failed'
                }
            }
        }
    }
}
```

## Generic shell

For any CI that runs shell commands:

```sh
# Install
curl -fsSL https://github.com/alinaqi2000/lockguard/releases/latest/download/install.sh | sh

# Audit — exit 1 if vulnerabilities found
lockguard --format json --min-severity high > audit-report.json

code=$?
if [ $code -eq 0 ]; then
    echo "No vulnerabilities found"
elif [ $code -eq 1 ]; then
    echo "Vulnerabilities found — see audit-report.json"
    jq '.findings[] | .package + " " + .severity' audit-report.json
    exit 1
else
    echo "Audit failed (exit code $code)"
    exit 1
fi
```

## Tips

- **Use `--format json`** when capturing output programmatically. Text format is for humans.
- **Use `--min-severity`** to control what blocks the pipeline. A common policy: block on `high` and `critical`, report `medium` and `low` as warnings.
- **Cache the binary** if your CI supports caching. The install script downloads it each time otherwise.
- **Don't parse stderr** for results. All report data is on stdout. stderr contains only progress and warnings.
- **Pin a version** in production CI to avoid surprises from new releases. Replace `latest` with a specific tag like `v0.1.4`.
