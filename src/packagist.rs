use crate::error::{Error, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;

const BATCH_SIZE: usize = 50;
const USER_AGENT: &str = "lockguard/0.1.0";
pub const DEFAULT_BASE_URL: &str = "https://packagist.org";

#[derive(Deserialize, Debug)]
pub struct AdvisoryResponse {
    pub advisories: HashMap<String, Vec<WireAdvisory>>,
}

#[derive(Deserialize, Debug)]
pub struct WireAdvisory {
    #[serde(rename = "advisoryId")]
    pub advisory_id: String,
    #[serde(rename = "packageName")]
    pub package_name: String,
    pub title: String,
    pub link: Option<String>,
    pub cve: Option<String>,
    #[serde(rename = "affectedVersions")]
    pub affected_versions: String,
    pub sources: Option<Vec<WireSource>>,
    pub severity: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct WireSource {
    pub name: String,
    #[serde(rename = "remoteId")]
    pub remote_id: String,
}

pub struct Client {
    http: reqwest::Client,
    base_url: String,
}

impl Client {
    pub fn new() -> Result<Self> {
        Self::with_base_url(DEFAULT_BASE_URL)
    }

    pub fn with_base_url(base_url: &str) -> Result<Self> {
        let http = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| Error::ClientBuild(e.to_string()))?;

        Ok(Self {
            http,
            base_url: base_url.trim_end_matches('/').to_string(),
        })
    }

    pub async fn fetch_advisories(&self, package_names: &[String]) -> Result<AdvisoryResponse> {
        let mut sorted: Vec<&String> = package_names.iter().collect();
        sorted.sort();
        sorted.dedup();

        let mut merged: HashMap<String, Vec<WireAdvisory>> = HashMap::new();

        for chunk in sorted.chunks(BATCH_SIZE) {
            let response = self.fetch_batch(chunk).await?;
            for (name, advisories) in response.advisories {
                merged.entry(name).or_default().extend(advisories);
            }
        }

        Ok(AdvisoryResponse { advisories: merged })
    }

    async fn fetch_batch(&self, packages: &[&String]) -> Result<AdvisoryResponse> {
        let mut url =
            reqwest::Url::parse(&self.base_url).map_err(|e| Error::ClientBuild(e.to_string()))?;
        url.path_segments_mut()
            .map_err(|_| Error::ClientBuild("invalid base URL path".to_string()))?
            .extend(&["api", "security-advisories"]);

        for pkg in packages {
            url.query_pairs_mut().append_pair("packages[]", pkg);
        }

        let response = self.http.get(url.clone()).send().await?;

        let status = response.status();
        if !status.is_success() {
            return Err(Error::HttpStatus {
                status: status.as_u16(),
                url: url.to_string(),
            });
        }

        let body = response.text().await?;
        let parsed: AdvisoryResponse =
            serde_json::from_str(&body).map_err(Error::ResponseDecode)?;

        Ok(parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn sample_advisory_json() -> &'static str {
        r#"{
            "advisories": {
                "monolog/monolog": [
                    {
                        "advisoryId": "PKSA-dmw8-jd8k-q3c6",
                        "packageName": "monolog/monolog",
                        "remoteId": "monolog/monolog/2014-12-29-1.yaml",
                        "title": "Header injection in NativeMailerHandler",
                        "link": "https://github.com/Seldaek/monolog/pull/448",
                        "cve": null,
                        "affectedVersions": ">=1.8.0,<1.12.0",
                        "source": "FriendsOfPHP/security-advisories",
                        "sources": [
                            {"name": "GitHub", "remoteId": "GHSA-f57v-q966-7fh6"},
                            {"name": "FriendsOfPHP/security-advisories", "remoteId": "monolog/monolog/2014-12-29-1.yaml"}
                        ],
                        "reportedAt": "2014-12-29 00:00:00",
                        "composerRepository": "https://packagist.org",
                        "severity": "low"
                    }
                ]
            }
        }"#
    }

    #[test]
    fn deserialize_advisory_with_all_fields() {
        let response: AdvisoryResponse = serde_json::from_str(sample_advisory_json()).unwrap();
        let advisories = response.advisories.get("monolog/monolog").unwrap();
        assert_eq!(advisories.len(), 1);
        let a = &advisories[0];
        assert_eq!(a.advisory_id, "PKSA-dmw8-jd8k-q3c6");
        assert_eq!(a.package_name, "monolog/monolog");
        assert_eq!(a.title, "Header injection in NativeMailerHandler");
        assert!(a.cve.is_none());
        assert_eq!(a.affected_versions, ">=1.8.0,<1.12.0");
        assert_eq!(a.severity.as_deref(), Some("low"));
        let sources = a.sources.as_ref().unwrap();
        assert_eq!(sources.len(), 2);
        assert_eq!(sources[0].name, "GitHub");
        assert_eq!(sources[0].remote_id, "GHSA-f57v-q966-7fh6");
    }

    #[test]
    fn deserialize_empty_advisories() {
        let json = r#"{"advisories": {}}"#;
        let response: AdvisoryResponse = serde_json::from_str(json).unwrap();
        assert!(response.advisories.is_empty());
    }

    #[test]
    fn deserialize_known_clean_package() {
        let json = r#"{"advisories": {"vendor/clean": []}}"#;
        let response: AdvisoryResponse = serde_json::from_str(json).unwrap();
        let advisories = response.advisories.get("vendor/clean").unwrap();
        assert!(advisories.is_empty());
    }

    #[test]
    fn deserialize_null_severity() {
        let json = r#"{
            "advisories": {
                "vendor/pkg": [{
                    "advisoryId": "PKSA-test",
                    "packageName": "vendor/pkg",
                    "title": "Test",
                    "link": null,
                    "cve": "CVE-2024-1234",
                    "affectedVersions": ">=1.0.0",
                    "sources": null,
                    "severity": null
                }]
            }
        }"#;
        let response: AdvisoryResponse = serde_json::from_str(json).unwrap();
        let a = &response.advisories["vendor/pkg"][0];
        assert!(a.severity.is_none());
        assert!(a.link.is_none());
        assert!(a.sources.is_none());
        assert_eq!(a.cve.as_deref(), Some("CVE-2024-1234"));
    }

    #[test]
    fn deserialize_missing_optional_fields() {
        let json = r#"{
            "advisories": {
                "vendor/pkg": [{
                    "advisoryId": "PKSA-test",
                    "packageName": "vendor/pkg",
                    "title": "Test",
                    "affectedVersions": ">=1.0.0"
                }]
            }
        }"#;
        let response: AdvisoryResponse = serde_json::from_str(json).unwrap();
        let a = &response.advisories["vendor/pkg"][0];
        assert!(a.cve.is_none());
        assert!(a.link.is_none());
        assert!(a.sources.is_none());
        assert!(a.severity.is_none());
    }

    #[test]
    fn deserialize_multiple_advisories_per_package() {
        let json = r#"{
            "advisories": {
                "vendor/pkg": [
                    {"advisoryId": "PKSA-1", "packageName": "vendor/pkg", "title": "First", "affectedVersions": ">=1.0.0,<2.0.0"},
                    {"advisoryId": "PKSA-2", "packageName": "vendor/pkg", "title": "Second", "affectedVersions": ">=2.0.0,<3.0.0"}
                ]
            }
        }"#;
        let response: AdvisoryResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.advisories["vendor/pkg"].len(), 2);
    }

    #[test]
    fn deserialize_unknown_additive_fields_tolerated() {
        let json = r#"{
            "advisories": {
                "vendor/pkg": [{
                    "advisoryId": "PKSA-test",
                    "packageName": "vendor/pkg",
                    "title": "Test",
                    "affectedVersions": ">=1.0.0",
                    "futureField": "ignored",
                    "anotherNewField": 42
                }]
            }
        }"#;
        let response: AdvisoryResponse = serde_json::from_str(json).unwrap();
        assert_eq!(
            response.advisories["vendor/pkg"][0].advisory_id,
            "PKSA-test"
        );
    }

    #[test]
    fn deserialize_missing_required_field_rejected() {
        let json = r#"{
            "advisories": {
                "vendor/pkg": [{
                    "packageName": "vendor/pkg",
                    "title": "Test",
                    "affectedVersions": ">=1.0.0"
                }]
            }
        }"#;
        assert!(serde_json::from_str::<AdvisoryResponse>(json).is_err());
    }

    #[tokio::test]
    async fn fetch_advisories_success() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/security-advisories"))
            .and(query_param("packages[]", "monolog/monolog"))
            .respond_with(ResponseTemplate::new(200).set_body_string(sample_advisory_json()))
            .mount(&server)
            .await;

        let client = Client::with_base_url(&server.uri()).unwrap();
        let response = client
            .fetch_advisories(&["monolog/monolog".to_string()])
            .await
            .unwrap();

        assert!(response.advisories.contains_key("monolog/monolog"));
    }

    #[tokio::test]
    async fn fetch_advisories_multiple_packages() {
        let server = MockServer::start().await;
        let json = r#"{
            "advisories": {
                "vendor/a": [{"advisoryId": "PKSA-a", "packageName": "vendor/a", "title": "A", "affectedVersions": ">=1.0.0"}],
                "vendor/b": [{"advisoryId": "PKSA-b", "packageName": "vendor/b", "title": "B", "affectedVersions": ">=2.0.0"}]
            }
        }"#;
        Mock::given(method("GET"))
            .and(path("/api/security-advisories"))
            .respond_with(ResponseTemplate::new(200).set_body_string(json))
            .mount(&server)
            .await;

        let client = Client::with_base_url(&server.uri()).unwrap();
        let response = client
            .fetch_advisories(&["vendor/a".to_string(), "vendor/b".to_string()])
            .await
            .unwrap();

        assert!(response.advisories.contains_key("vendor/a"));
        assert!(response.advisories.contains_key("vendor/b"));
    }

    #[tokio::test]
    async fn fetch_advisories_404_status() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/security-advisories"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&server)
            .await;

        let client = Client::with_base_url(&server.uri()).unwrap();
        let err = client
            .fetch_advisories(&["vendor/pkg".to_string()])
            .await
            .unwrap_err();
        assert!(matches!(err, Error::HttpStatus { status: 404, .. }));
    }

    #[tokio::test]
    async fn fetch_advisories_500_status() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/security-advisories"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&server)
            .await;

        let client = Client::with_base_url(&server.uri()).unwrap();
        let err = client
            .fetch_advisories(&["vendor/pkg".to_string()])
            .await
            .unwrap_err();
        assert!(matches!(err, Error::HttpStatus { status: 500, .. }));
    }

    #[tokio::test]
    async fn fetch_advisories_429_status() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/security-advisories"))
            .respond_with(ResponseTemplate::new(429))
            .mount(&server)
            .await;

        let client = Client::with_base_url(&server.uri()).unwrap();
        let err = client
            .fetch_advisories(&["vendor/pkg".to_string()])
            .await
            .unwrap_err();
        assert!(matches!(err, Error::HttpStatus { status: 429, .. }));
    }

    #[tokio::test]
    async fn fetch_advisories_malformed_json() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/security-advisories"))
            .respond_with(ResponseTemplate::new(200).set_body_string("not json"))
            .mount(&server)
            .await;

        let client = Client::with_base_url(&server.uri()).unwrap();
        let err = client
            .fetch_advisories(&["vendor/pkg".to_string()])
            .await
            .unwrap_err();
        assert!(matches!(err, Error::ResponseDecode(_)));
    }

    #[tokio::test]
    async fn fetch_advisories_empty_response() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/security-advisories"))
            .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"advisories": {}}"#))
            .mount(&server)
            .await;

        let client = Client::with_base_url(&server.uri()).unwrap();
        let response = client
            .fetch_advisories(&["vendor/pkg".to_string()])
            .await
            .unwrap();
        assert!(response.advisories.is_empty());
    }

    #[tokio::test]
    async fn fetch_advisories_batches_large_input() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/security-advisories"))
            .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"advisories": {}}"#))
            .mount(&server)
            .await;

        let client = Client::with_base_url(&server.uri()).unwrap();
        let packages: Vec<String> = (0..120).map(|i| format!("vendor/pkg{i}")).collect();

        let response = client.fetch_advisories(&packages).await.unwrap();
        assert!(response.advisories.is_empty());
    }

    #[tokio::test]
    async fn fetch_advisories_dedup_packages() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/security-advisories"))
            .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"advisories": {}}"#))
            .expect(1)
            .mount(&server)
            .await;

        let client = Client::with_base_url(&server.uri()).unwrap();
        let packages = vec!["vendor/pkg".to_string(), "vendor/pkg".to_string()];
        client.fetch_advisories(&packages).await.unwrap();
    }

    #[tokio::test]
    async fn fetch_advisories_sorts_packages() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/security-advisories"))
            .and(query_param("packages[]", "aaa/first"))
            .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"advisories": {}}"#))
            .mount(&server)
            .await;

        let client = Client::with_base_url(&server.uri()).unwrap();
        let packages = vec!["zzz/last".to_string(), "aaa/first".to_string()];
        client.fetch_advisories(&packages).await.unwrap();
    }
}
