use lockguard::cli::{Cli, Format, SeverityArg};
use lockguard::run_with_base_url;
use std::path::PathBuf;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn fixture_lock(name: &str) -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("tests/fixtures/locks");
    p.push(name);
    p
}

fn fixture_response(name: &str) -> String {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("tests/fixtures/packagist");
    p.push(name);
    std::fs::read_to_string(p).unwrap()
}

fn cli(lock: &str) -> Cli {
    Cli {
        lock: fixture_lock(lock),
        format: Format::Text,
        min_severity: SeverityArg::Low,
    }
}

async fn setup_mock(server: &MockServer, response_file: &str) {
    let body = fixture_response(response_file);
    Mock::given(method("GET"))
        .and(path("/api/security-advisories"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(server)
        .await;
}

#[tokio::test]
async fn clean_audit_exit_zero() {
    let server = MockServer::start().await;
    setup_mock(&server, "clean.json").await;

    let result = run_with_base_url(cli("clean.lock"), &server.uri()).await;
    assert!(result.is_ok());
    let code = result.unwrap();
    assert_eq!(code, std::process::ExitCode::SUCCESS);
}

#[tokio::test]
async fn vulnerable_audit_exit_one() {
    let server = MockServer::start().await;
    setup_mock(&server, "vulnerable.json").await;

    let result = run_with_base_url(cli("vulnerable.lock"), &server.uri()).await;
    assert!(result.is_ok());
    let code = result.unwrap();
    assert_eq!(code, std::process::ExitCode::from(1));
}

#[tokio::test]
async fn empty_lock_exit_zero() {
    let server = MockServer::start().await;
    setup_mock(&server, "clean.json").await;

    let result = run_with_base_url(cli("empty.lock"), &server.uri()).await;
    assert!(result.is_ok());
    let code = result.unwrap();
    assert_eq!(code, std::process::ExitCode::SUCCESS);
}

#[tokio::test]
async fn malformed_lock_exit_two() {
    let server = MockServer::start().await;
    setup_mock(&server, "clean.json").await;

    let result = run_with_base_url(cli("malformed.lock"), &server.uri()).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn missing_lock_exit_two() {
    let server = MockServer::start().await;
    setup_mock(&server, "clean.json").await;

    let mut c = cli("clean.lock");
    c.lock = PathBuf::from("/nonexistent/composer.lock");
    let result = run_with_base_url(c, &server.uri()).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn duplicate_conflict_exit_two() {
    let server = MockServer::start().await;
    setup_mock(&server, "clean.json").await;

    let result = run_with_base_url(cli("duplicate-conflict.lock"), &server.uri()).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn api_error_exit_two() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/security-advisories"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;

    let result = run_with_base_url(cli("clean.lock"), &server.uri()).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn json_format_clean() {
    let server = MockServer::start().await;
    setup_mock(&server, "clean.json").await;

    let mut c = cli("clean.lock");
    c.format = Format::Json;
    let result = run_with_base_url(c, &server.uri()).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), std::process::ExitCode::SUCCESS);
}

#[tokio::test]
async fn json_format_vulnerable() {
    let server = MockServer::start().await;
    setup_mock(&server, "vulnerable.json").await;

    let mut c = cli("vulnerable.lock");
    c.format = Format::Json;
    let result = run_with_base_url(c, &server.uri()).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), std::process::ExitCode::from(1));
}

#[tokio::test]
async fn severity_filter_excludes_low() {
    let server = MockServer::start().await;
    setup_mock(&server, "vulnerable.json").await;

    let mut c = cli("vulnerable.lock");
    c.min_severity = SeverityArg::Critical;
    let result = run_with_base_url(c, &server.uri()).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), std::process::ExitCode::SUCCESS);
}
