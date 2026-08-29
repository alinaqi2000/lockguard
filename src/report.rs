use crate::audit::{AuditResult, Finding, Severity};
use crate::error::{Error, Result};
use std::io::Write;

pub fn render_text(result: &AuditResult, out: &mut impl Write) -> Result<()> {
    if result.findings.is_empty() {
        writeln!(out, "No known vulnerabilities found.").map_err(io_err)?;
    } else {
        writeln!(
            out,
            "Found {} packages with known vulnerabilities.",
            result.vulnerable_packages
        )
        .map_err(io_err)?;
        writeln!(out).map_err(io_err)?;

        for finding in &result.findings {
            render_finding_text(finding, out)?;
            writeln!(out).map_err(io_err)?;
        }
    }

    writeln!(out, "Summary:").map_err(io_err)?;
    writeln!(out, "  - Total packages: {}", result.total_packages).map_err(io_err)?;
    writeln!(
        out,
        "  - Vulnerable packages: {}",
        result.vulnerable_packages
    )
    .map_err(io_err)?;
    writeln!(out, "  - Critical: {}", result.summary.critical).map_err(io_err)?;
    writeln!(out, "  - High: {}", result.summary.high).map_err(io_err)?;
    writeln!(out, "  - Medium: {}", result.summary.medium).map_err(io_err)?;
    writeln!(out, "  - Low: {}", result.summary.low).map_err(io_err)?;
    writeln!(out, "  - Unknown: {}", result.summary.unknown).map_err(io_err)?;

    Ok(())
}

fn render_finding_text(finding: &Finding, out: &mut impl Write) -> Result<()> {
    let label = severity_label(&finding.severity);
    writeln!(out, "[{}] {} {}", label, finding.package, finding.version).map_err(io_err)?;

    writeln!(out, "  - Advisory: {}", finding.advisory_id).map_err(io_err)?;

    if let Some(cve) = &finding.cve {
        writeln!(out, "  - CVE: {}", cve).map_err(io_err)?;
    }

    writeln!(out, "  - {}", finding.title).map_err(io_err)?;
    writeln!(out, "  - Affected: {}", finding.affected_versions).map_err(io_err)?;

    if let Some(link) = &finding.link {
        writeln!(out, "  - Link: {}", link).map_err(io_err)?;
    }

    if !finding.sources.is_empty() {
        let sources: Vec<String> = finding
            .sources
            .iter()
            .map(|s| format!("{}:{}", s.name, s.remote_id))
            .collect();
        writeln!(out, "  - Sources: {}", sources.join(", ")).map_err(io_err)?;
    }

    Ok(())
}

pub fn render_json(result: &AuditResult, out: &mut impl Write) -> Result<()> {
    let json =
        serde_json::to_string_pretty(result).map_err(|e| Error::ReportWrite(e.to_string()))?;
    writeln!(out, "{}", json).map_err(io_err)?;
    Ok(())
}

fn severity_label(s: &Severity) -> &'static str {
    match s {
        Severity::Critical => "CRITICAL",
        Severity::High => "HIGH",
        Severity::Medium => "MEDIUM",
        Severity::Low => "LOW",
        Severity::Unknown => "UNKNOWN",
    }
}

fn io_err(e: std::io::Error) -> Error {
    Error::ReportWrite(e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::{Finding, Source, Summary};

    fn make_result(findings: Vec<Finding>, total: usize) -> AuditResult {
        let vulnerable_packages = findings
            .iter()
            .map(|f| f.package.clone())
            .collect::<std::collections::HashSet<_>>()
            .len();

        let mut summary = Summary {
            critical: 0,
            high: 0,
            medium: 0,
            low: 0,
            unknown: 0,
        };
        for f in &findings {
            match f.severity {
                Severity::Critical => summary.critical += 1,
                Severity::High => summary.high += 1,
                Severity::Medium => summary.medium += 1,
                Severity::Low => summary.low += 1,
                Severity::Unknown => summary.unknown += 1,
            }
        }

        AuditResult {
            total_packages: total,
            vulnerable_packages,
            findings,
            summary,
            coverage_unknown: vec![],
        }
    }

    fn make_finding(
        package: &str,
        version: &str,
        severity: Severity,
        cve: Option<&str>,
        link: Option<&str>,
        sources: Vec<Source>,
    ) -> Finding {
        Finding {
            package: package.to_string(),
            version: version.to_string(),
            advisory_id: "PKSA-test".to_string(),
            severity,
            cve: cve.map(|s| s.to_string()),
            title: "Test vulnerability".to_string(),
            affected_versions: ">=1.0.0,<2.0.0".to_string(),
            link: link.map(|s| s.to_string()),
            sources,
        }
    }

    #[test]
    fn text_empty_report() {
        let result = make_result(vec![], 42);
        let mut out = Vec::new();
        render_text(&result, &mut out).unwrap();
        let text = String::from_utf8(out).unwrap();
        assert!(text.contains("No known vulnerabilities found."));
        assert!(text.contains("Total packages: 42"));
        assert!(text.contains("Vulnerable packages: 0"));
    }

    #[test]
    fn text_populated_report() {
        let result = make_result(
            vec![make_finding(
                "monolog/monolog",
                "2.3.0",
                Severity::High,
                Some("CVE-2023-1234"),
                Some("https://example.com"),
                vec![],
            )],
            42,
        );
        let mut out = Vec::new();
        render_text(&result, &mut out).unwrap();
        let text = String::from_utf8(out).unwrap();
        assert!(text.contains("Found 1 packages with known vulnerabilities."));
        assert!(text.contains("[HIGH] monolog/monolog 2.3.0"));
        assert!(text.contains("CVE: CVE-2023-1234"));
        assert!(text.contains("Link: https://example.com"));
        assert!(text.contains("High: 1"));
    }

    #[test]
    fn text_finding_without_optional_fields() {
        let result = make_result(
            vec![make_finding(
                "vendor/pkg",
                "1.0.0",
                Severity::Unknown,
                None,
                None,
                vec![],
            )],
            1,
        );
        let mut out = Vec::new();
        render_text(&result, &mut out).unwrap();
        let text = String::from_utf8(out).unwrap();
        assert!(text.contains("[UNKNOWN] vendor/pkg 1.0.0"));
        assert!(!text.contains("CVE:"));
        assert!(!text.contains("Link:"));
        assert!(!text.contains("Sources:"));
    }

    #[test]
    fn text_finding_with_sources() {
        let result = make_result(
            vec![make_finding(
                "vendor/pkg",
                "1.0.0",
                Severity::Medium,
                None,
                None,
                vec![
                    Source {
                        name: "GitHub".to_string(),
                        remote_id: "GHSA-abc".to_string(),
                    },
                    Source {
                        name: "FriendsOfPHP".to_string(),
                        remote_id: "vendor/pkg/2024.yaml".to_string(),
                    },
                ],
            )],
            1,
        );
        let mut out = Vec::new();
        render_text(&result, &mut out).unwrap();
        let text = String::from_utf8(out).unwrap();
        assert!(text.contains("Sources: GitHub:GHSA-abc, FriendsOfPHP:vendor/pkg/2024.yaml"));
    }

    #[test]
    fn json_empty_report() {
        let result = make_result(vec![], 42);
        let mut out = Vec::new();
        render_json(&result, &mut out).unwrap();
        let json = String::from_utf8(out).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["total_packages"], 42);
        assert_eq!(parsed["vulnerable_packages"], 0);
        assert_eq!(parsed["findings"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn json_populated_report() {
        let result = make_result(
            vec![make_finding(
                "monolog/monolog",
                "2.3.0",
                Severity::High,
                Some("CVE-2023-1234"),
                Some("https://example.com"),
                vec![Source {
                    name: "GitHub".to_string(),
                    remote_id: "GHSA-abc".to_string(),
                }],
            )],
            42,
        );
        let mut out = Vec::new();
        render_json(&result, &mut out).unwrap();
        let json = String::from_utf8(out).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["total_packages"], 42);
        assert_eq!(parsed["vulnerable_packages"], 1);
        assert_eq!(parsed["findings"][0]["package"], "monolog/monolog");
        assert_eq!(parsed["findings"][0]["version"], "2.3.0");
        assert_eq!(parsed["findings"][0]["advisory_id"], "PKSA-test");
        assert_eq!(parsed["findings"][0]["severity"], "high");
        assert_eq!(parsed["findings"][0]["cve"], "CVE-2023-1234");
        assert_eq!(parsed["findings"][0]["title"], "Test vulnerability");
        assert_eq!(parsed["findings"][0]["affected_versions"], ">=1.0.0,<2.0.0");
        assert_eq!(parsed["findings"][0]["link"], "https://example.com");
        assert_eq!(parsed["findings"][0]["sources"][0]["name"], "GitHub");
        assert_eq!(parsed["findings"][0]["sources"][0]["remote_id"], "GHSA-abc");
    }

    #[test]
    fn json_null_fields_serialize_as_null() {
        let result = make_result(
            vec![make_finding(
                "vendor/pkg",
                "1.0.0",
                Severity::Unknown,
                None,
                None,
                vec![],
            )],
            1,
        );
        let mut out = Vec::new();
        render_json(&result, &mut out).unwrap();
        let json = String::from_utf8(out).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert!(parsed["findings"][0]["cve"].is_null());
        assert!(parsed["findings"][0]["link"].is_null());
        assert_eq!(
            parsed["findings"][0]["sources"].as_array().unwrap().len(),
            0
        );
    }

    #[test]
    fn json_and_text_summaries_agree() {
        let result = make_result(
            vec![
                make_finding("vendor/a", "1.0.0", Severity::High, None, None, vec![]),
                make_finding("vendor/b", "2.0.0", Severity::Medium, None, None, vec![]),
            ],
            10,
        );

        let mut text_out = Vec::new();
        render_text(&result, &mut text_out).unwrap();
        let text = String::from_utf8(text_out).unwrap();

        let mut json_out = Vec::new();
        render_json(&result, &mut json_out).unwrap();
        let json = String::from_utf8(json_out).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert!(text.contains("Vulnerable packages: 2"));
        assert_eq!(parsed["vulnerable_packages"], 2);
        assert!(text.contains("High: 1"));
        assert_eq!(parsed["summary"]["high"], 1);
        assert!(text.contains("Medium: 1"));
        assert_eq!(parsed["summary"]["medium"], 1);
    }

    #[test]
    fn json_stable_across_repeated_runs() {
        let result = make_result(
            vec![
                make_finding("vendor/a", "1.0.0", Severity::High, None, None, vec![]),
                make_finding("vendor/b", "2.0.0", Severity::Low, None, None, vec![]),
            ],
            10,
        );

        let mut out1 = Vec::new();
        render_json(&result, &mut out1).unwrap();
        let json1 = String::from_utf8(out1).unwrap();

        let mut out2 = Vec::new();
        render_json(&result, &mut out2).unwrap();
        let json2 = String::from_utf8(out2).unwrap();

        assert_eq!(json1, json2);
    }

    #[test]
    fn json_no_coverage_unknown_field() {
        let result = AuditResult {
            total_packages: 1,
            vulnerable_packages: 0,
            findings: vec![],
            summary: Summary {
                critical: 0,
                high: 0,
                medium: 0,
                low: 0,
                unknown: 0,
            },
            coverage_unknown: vec!["vendor/unknown".to_string()],
        };

        let mut out = Vec::new();
        render_json(&result, &mut out).unwrap();
        let json = String::from_utf8(out).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed.get("coverage_unknown").is_none());
    }
}
