use crate::cli::SeverityArg;
use crate::lock::LockedPackage;
use crate::packagist::{AdvisoryResponse, WireAdvisory};
use composer_semver::{Constraint, Version};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Unknown,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize)]
pub struct Source {
    pub name: String,
    pub remote_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Finding {
    pub package: String,
    pub version: String,
    pub advisory_id: String,
    pub severity: Severity,
    pub cve: Option<String>,
    pub title: String,
    pub affected_versions: String,
    pub link: Option<String>,
    pub sources: Vec<Source>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Summary {
    pub critical: usize,
    pub high: usize,
    pub medium: usize,
    pub low: usize,
    pub unknown: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuditResult {
    pub total_packages: usize,
    pub vulnerable_packages: usize,
    pub findings: Vec<Finding>,
    pub summary: Summary,
    #[serde(skip)]
    pub coverage_unknown: Vec<String>,
}

pub fn normalize_severity(raw: Option<&str>) -> Severity {
    match raw {
        Some(s) => match s.to_lowercase().as_str() {
            "low" => Severity::Low,
            "medium" => Severity::Medium,
            "high" => Severity::High,
            "critical" => Severity::Critical,
            _ => Severity::Unknown,
        },
        None => Severity::Unknown,
    }
}

pub fn meets_threshold(finding: &Severity, threshold: SeverityArg) -> bool {
    let finding_level = match finding {
        Severity::Critical => 4,
        Severity::High => 3,
        Severity::Medium => 2,
        Severity::Low => 1,
        Severity::Unknown => 1,
    };
    let threshold_level = match threshold {
        SeverityArg::Low => 1,
        SeverityArg::Medium => 2,
        SeverityArg::High => 3,
        SeverityArg::Critical => 4,
    };
    finding_level >= threshold_level
}

pub fn audit(
    packages: &[LockedPackage],
    response: &AdvisoryResponse,
    min_severity: SeverityArg,
) -> AuditResult {
    let mut findings: Vec<Finding> = Vec::new();
    let mut coverage_unknown: Vec<String> = Vec::new();

    for pkg in packages {
        match response.advisories.get(&pkg.name) {
            Some(advisories) if advisories.is_empty() => {}
            Some(advisories) => {
                for advisory in advisories {
                    if let Some(finding) = evaluate_advisory(pkg, advisory) {
                        if meets_threshold(&finding.severity, min_severity) {
                            findings.push(finding);
                        }
                    }
                }
            }
            None => {
                coverage_unknown.push(pkg.name.clone());
            }
        }
    }

    sort_findings(&mut findings);

    let vulnerable_packages = findings
        .iter()
        .map(|f| &f.package)
        .collect::<std::collections::HashSet<_>>()
        .len();

    let summary = build_summary(&findings);

    AuditResult {
        total_packages: packages.len(),
        vulnerable_packages,
        findings,
        summary,
        coverage_unknown,
    }
}

fn evaluate_advisory(pkg: &LockedPackage, advisory: &WireAdvisory) -> Option<Finding> {
    let version = match Version::parse(&pkg.version) {
        Ok(v) => v,
        Err(_) => {
            eprintln!(
                "warning: cannot parse version '{}' for {} — skipping advisory {}",
                pkg.version, pkg.name, advisory.advisory_id
            );
            return None;
        }
    };

    let constraint = match Constraint::parse(&advisory.affected_versions) {
        Ok(c) => c,
        Err(_) => {
            eprintln!(
                "warning: cannot parse constraint '{}' for {} — skipping advisory {}",
                advisory.affected_versions, pkg.name, advisory.advisory_id
            );
            return None;
        }
    };

    if !constraint.matches(&version) {
        return None;
    }

    let sources = advisory
        .sources
        .as_ref()
        .map(|s| {
            s.iter()
                .map(|src| Source {
                    name: src.name.clone(),
                    remote_id: src.remote_id.clone(),
                })
                .collect()
        })
        .unwrap_or_default();

    Some(Finding {
        package: pkg.name.clone(),
        version: pkg.version.clone(),
        advisory_id: advisory.advisory_id.clone(),
        severity: normalize_severity(advisory.severity.as_deref()),
        cve: advisory.cve.clone(),
        title: advisory.title.clone(),
        affected_versions: advisory.affected_versions.clone(),
        link: advisory.link.clone(),
        sources,
    })
}

fn sort_findings(findings: &mut [Finding]) {
    findings.sort_by(|a, b| {
        b.severity
            .cmp(&a.severity)
            .then_with(|| a.package.cmp(&b.package))
            .then_with(|| a.version.cmp(&b.version))
            .then_with(|| a.advisory_id.cmp(&b.advisory_id))
    });
}

fn build_summary(findings: &[Finding]) -> Summary {
    let mut summary = Summary {
        critical: 0,
        high: 0,
        medium: 0,
        low: 0,
        unknown: 0,
    };
    for f in findings {
        match f.severity {
            Severity::Critical => summary.critical += 1,
            Severity::High => summary.high += 1,
            Severity::Medium => summary.medium += 1,
            Severity::Low => summary.low += 1,
            Severity::Unknown => summary.unknown += 1,
        }
    }
    summary
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lock::LockedPackage;
    use std::collections::HashMap;

    fn pkg(name: &str, version: &str) -> LockedPackage {
        LockedPackage {
            name: name.to_string(),
            version: version.to_string(),
        }
    }

    fn wire_advisory(id: &str, affected: &str, severity: Option<&str>) -> WireAdvisory {
        WireAdvisory {
            advisory_id: id.to_string(),
            package_name: "test/package".to_string(),
            title: "Test advisory".to_string(),
            link: None,
            cve: None,
            affected_versions: affected.to_string(),
            sources: None,
            severity: severity.map(|s| s.to_string()),
        }
    }

    fn response_with(advisories: HashMap<String, Vec<WireAdvisory>>) -> AdvisoryResponse {
        AdvisoryResponse { advisories }
    }

    // --- Severity normalization ---

    #[test]
    fn severity_low() {
        assert_eq!(normalize_severity(Some("low")), Severity::Low);
    }

    #[test]
    fn severity_medium() {
        assert_eq!(normalize_severity(Some("medium")), Severity::Medium);
    }

    #[test]
    fn severity_high() {
        assert_eq!(normalize_severity(Some("high")), Severity::High);
    }

    #[test]
    fn severity_critical() {
        assert_eq!(normalize_severity(Some("critical")), Severity::Critical);
    }

    #[test]
    fn severity_case_insensitive() {
        assert_eq!(normalize_severity(Some("HIGH")), Severity::High);
        assert_eq!(normalize_severity(Some("Critical")), Severity::Critical);
    }

    #[test]
    fn severity_unknown_string() {
        assert_eq!(normalize_severity(Some("bogus")), Severity::Unknown);
    }

    #[test]
    fn severity_none() {
        assert_eq!(normalize_severity(None), Severity::Unknown);
    }

    // --- Threshold matrix ---

    #[test]
    fn threshold_low_includes_all() {
        assert!(meets_threshold(&Severity::Critical, SeverityArg::Low));
        assert!(meets_threshold(&Severity::High, SeverityArg::Low));
        assert!(meets_threshold(&Severity::Medium, SeverityArg::Low));
        assert!(meets_threshold(&Severity::Low, SeverityArg::Low));
        assert!(meets_threshold(&Severity::Unknown, SeverityArg::Low));
    }

    #[test]
    fn threshold_medium_excludes_low_and_unknown() {
        assert!(meets_threshold(&Severity::Critical, SeverityArg::Medium));
        assert!(meets_threshold(&Severity::High, SeverityArg::Medium));
        assert!(meets_threshold(&Severity::Medium, SeverityArg::Medium));
        assert!(!meets_threshold(&Severity::Low, SeverityArg::Medium));
        assert!(!meets_threshold(&Severity::Unknown, SeverityArg::Medium));
    }

    #[test]
    fn threshold_high_excludes_low_medium_unknown() {
        assert!(meets_threshold(&Severity::Critical, SeverityArg::High));
        assert!(meets_threshold(&Severity::High, SeverityArg::High));
        assert!(!meets_threshold(&Severity::Medium, SeverityArg::High));
        assert!(!meets_threshold(&Severity::Low, SeverityArg::High));
        assert!(!meets_threshold(&Severity::Unknown, SeverityArg::High));
    }

    #[test]
    fn threshold_critical_only_critical() {
        assert!(meets_threshold(&Severity::Critical, SeverityArg::Critical));
        assert!(!meets_threshold(&Severity::High, SeverityArg::Critical));
        assert!(!meets_threshold(&Severity::Medium, SeverityArg::Critical));
        assert!(!meets_threshold(&Severity::Low, SeverityArg::Critical));
        assert!(!meets_threshold(&Severity::Unknown, SeverityArg::Critical));
    }

    // --- Version matching ---

    #[test]
    fn match_version_inside_range() {
        let mut advisories = HashMap::new();
        advisories.insert(
            "vendor/pkg".to_string(),
            vec![wire_advisory("PKSA-1", ">=2.0.0,<2.4.0", Some("high"))],
        );
        let response = response_with(advisories);
        let packages = vec![pkg("vendor/pkg", "2.3.0")];

        let result = audit(&packages, &response, SeverityArg::Low);

        assert_eq!(result.findings.len(), 1);
        assert_eq!(result.findings[0].package, "vendor/pkg");
        assert_eq!(result.findings[0].version, "2.3.0");
    }

    #[test]
    fn match_version_outside_range() {
        let mut advisories = HashMap::new();
        advisories.insert(
            "vendor/pkg".to_string(),
            vec![wire_advisory("PKSA-1", ">=2.0.0,<2.4.0", Some("high"))],
        );
        let response = response_with(advisories);
        let packages = vec![pkg("vendor/pkg", "2.5.0")];

        let result = audit(&packages, &response, SeverityArg::Low);

        assert!(result.findings.is_empty());
        assert_eq!(result.vulnerable_packages, 0);
    }

    #[test]
    fn match_caret_constraint() {
        let mut advisories = HashMap::new();
        advisories.insert(
            "vendor/pkg".to_string(),
            vec![wire_advisory("PKSA-1", "^1.2", Some("medium"))],
        );
        let response = response_with(advisories);

        let matching = audit(&[pkg("vendor/pkg", "1.3.0")], &response, SeverityArg::Low);
        assert_eq!(matching.findings.len(), 1);

        let non_matching = audit(&[pkg("vendor/pkg", "2.0.0")], &response, SeverityArg::Low);
        assert!(non_matching.findings.is_empty());
    }

    #[test]
    fn match_tilde_constraint() {
        let mut advisories = HashMap::new();
        advisories.insert(
            "vendor/pkg".to_string(),
            vec![wire_advisory("PKSA-1", "~1.2", Some("low"))],
        );
        let response = response_with(advisories);

        // ~1.2 means >=1.2.0,<2.0.0 in Composer semantics
        let matching = audit(&[pkg("vendor/pkg", "1.2.9")], &response, SeverityArg::Low);
        assert_eq!(matching.findings.len(), 1);

        let matching_next_minor = audit(&[pkg("vendor/pkg", "1.3.0")], &response, SeverityArg::Low);
        assert_eq!(matching_next_minor.findings.len(), 1);

        let non_matching = audit(&[pkg("vendor/pkg", "2.0.0")], &response, SeverityArg::Low);
        assert!(non_matching.findings.is_empty());
    }

    #[test]
    fn match_wildcard_constraint() {
        let mut advisories = HashMap::new();
        advisories.insert(
            "vendor/pkg".to_string(),
            vec![wire_advisory("PKSA-1", "*", Some("high"))],
        );
        let response = response_with(advisories);

        let result = audit(
            &[pkg("vendor/pkg", "99.99.99")],
            &response,
            SeverityArg::Low,
        );
        assert_eq!(result.findings.len(), 1);
    }

    #[test]
    fn match_or_constraint() {
        let mut advisories = HashMap::new();
        advisories.insert(
            "vendor/pkg".to_string(),
            vec![wire_advisory(
                "PKSA-1",
                ">=1.0.0,<2.0.0||>=3.0.0,<4.0.0",
                Some("high"),
            )],
        );
        let response = response_with(advisories);

        let v1 = audit(&[pkg("vendor/pkg", "1.5.0")], &response, SeverityArg::Low);
        assert_eq!(v1.findings.len(), 1);

        let v2 = audit(&[pkg("vendor/pkg", "2.5.0")], &response, SeverityArg::Low);
        assert!(v2.findings.is_empty());

        let v3 = audit(&[pkg("vendor/pkg", "3.5.0")], &response, SeverityArg::Low);
        assert_eq!(v3.findings.len(), 1);
    }

    #[test]
    fn match_v_prefix_version() {
        let mut advisories = HashMap::new();
        advisories.insert(
            "vendor/pkg".to_string(),
            vec![wire_advisory("PKSA-1", ">=1.0.0,<2.0.0", Some("high"))],
        );
        let response = response_with(advisories);

        let result = audit(&[pkg("vendor/pkg", "v1.5.0")], &response, SeverityArg::Low);
        assert_eq!(result.findings.len(), 1);
    }

    #[test]
    fn match_boundary_at_lower_bound() {
        let mut advisories = HashMap::new();
        advisories.insert(
            "vendor/pkg".to_string(),
            vec![wire_advisory("PKSA-1", ">=2.0.0,<2.4.0", Some("high"))],
        );
        let response = response_with(advisories);

        let at_bound = audit(&[pkg("vendor/pkg", "2.0.0")], &response, SeverityArg::Low);
        assert_eq!(at_bound.findings.len(), 1);
    }

    #[test]
    fn match_boundary_at_upper_bound() {
        let mut advisories = HashMap::new();
        advisories.insert(
            "vendor/pkg".to_string(),
            vec![wire_advisory("PKSA-1", ">=2.0.0,<2.4.0", Some("high"))],
        );
        let response = response_with(advisories);

        let at_bound = audit(&[pkg("vendor/pkg", "2.4.0")], &response, SeverityArg::Low);
        assert!(at_bound.findings.is_empty());
    }

    #[test]
    fn match_multiple_advisories_one_package() {
        let mut advisories = HashMap::new();
        advisories.insert(
            "vendor/pkg".to_string(),
            vec![
                wire_advisory("PKSA-1", ">=1.0.0,<2.0.0", Some("high")),
                wire_advisory("PKSA-2", ">=1.5.0,<3.0.0", Some("medium")),
            ],
        );
        let response = response_with(advisories);

        let result = audit(&[pkg("vendor/pkg", "1.6.0")], &response, SeverityArg::Low);
        assert_eq!(result.findings.len(), 2);
        assert_eq!(result.vulnerable_packages, 1);
    }

    // --- Filtering ---

    #[test]
    fn filter_excludes_below_threshold() {
        let mut advisories = HashMap::new();
        advisories.insert(
            "vendor/pkg".to_string(),
            vec![wire_advisory("PKSA-1", "*", Some("low"))],
        );
        let response = response_with(advisories);
        let packages = vec![pkg("vendor/pkg", "1.0.0")];

        let result = audit(&packages, &response, SeverityArg::High);
        assert!(result.findings.is_empty());
    }

    #[test]
    fn filter_includes_unknown_at_low_threshold() {
        let mut advisories = HashMap::new();
        advisories.insert(
            "vendor/pkg".to_string(),
            vec![wire_advisory("PKSA-1", "*", None)],
        );
        let response = response_with(advisories);
        let packages = vec![pkg("vendor/pkg", "1.0.0")];

        let result = audit(&packages, &response, SeverityArg::Low);
        assert_eq!(result.findings.len(), 1);
        assert_eq!(result.findings[0].severity, Severity::Unknown);
    }

    #[test]
    fn filter_excludes_unknown_at_high_threshold() {
        let mut advisories = HashMap::new();
        advisories.insert(
            "vendor/pkg".to_string(),
            vec![wire_advisory("PKSA-1", "*", None)],
        );
        let response = response_with(advisories);
        let packages = vec![pkg("vendor/pkg", "1.0.0")];

        let result = audit(&packages, &response, SeverityArg::High);
        assert!(result.findings.is_empty());
    }

    // --- Coverage ---

    #[test]
    fn coverage_unknown_recorded() {
        let response = response_with(HashMap::new());
        let packages = vec![pkg("vendor/unknown", "1.0.0")];

        let result = audit(&packages, &response, SeverityArg::Low);
        assert_eq!(result.coverage_unknown, vec!["vendor/unknown"]);
    }

    #[test]
    fn coverage_known_clean_not_recorded() {
        let mut advisories = HashMap::new();
        advisories.insert("vendor/clean".to_string(), vec![]);
        let response = response_with(advisories);
        let packages = vec![pkg("vendor/clean", "1.0.0")];

        let result = audit(&packages, &response, SeverityArg::Low);
        assert!(result.coverage_unknown.is_empty());
        assert!(result.findings.is_empty());
    }

    // --- Sorting ---

    #[test]
    fn sort_by_severity_descending() {
        let mut advisories = HashMap::new();
        advisories.insert(
            "vendor/low".to_string(),
            vec![wire_advisory("PKSA-low", "*", Some("low"))],
        );
        advisories.insert(
            "vendor/high".to_string(),
            vec![wire_advisory("PKSA-high", "*", Some("high"))],
        );
        advisories.insert(
            "vendor/crit".to_string(),
            vec![wire_advisory("PKSA-crit", "*", Some("critical"))],
        );
        let response = response_with(advisories);
        let packages = vec![
            pkg("vendor/low", "1.0.0"),
            pkg("vendor/high", "1.0.0"),
            pkg("vendor/crit", "1.0.0"),
        ];

        let result = audit(&packages, &response, SeverityArg::Low);
        assert_eq!(result.findings[0].severity, Severity::Critical);
        assert_eq!(result.findings[1].severity, Severity::High);
        assert_eq!(result.findings[2].severity, Severity::Low);
    }

    #[test]
    fn sort_same_severity_by_package_name() {
        let mut advisories = HashMap::new();
        advisories.insert(
            "vendor/zzz".to_string(),
            vec![wire_advisory("PKSA-z", "*", Some("high"))],
        );
        advisories.insert(
            "vendor/aaa".to_string(),
            vec![wire_advisory("PKSA-a", "*", Some("high"))],
        );
        let response = response_with(advisories);
        let packages = vec![pkg("vendor/zzz", "1.0.0"), pkg("vendor/aaa", "1.0.0")];

        let result = audit(&packages, &response, SeverityArg::Low);
        assert_eq!(result.findings[0].package, "vendor/aaa");
        assert_eq!(result.findings[1].package, "vendor/zzz");
    }

    // --- Summary ---

    #[test]
    fn summary_counts_correct() {
        let mut advisories = HashMap::new();
        advisories.insert(
            "vendor/a".to_string(),
            vec![wire_advisory("PKSA-1", "*", Some("critical"))],
        );
        advisories.insert(
            "vendor/b".to_string(),
            vec![wire_advisory("PKSA-2", "*", Some("high"))],
        );
        advisories.insert(
            "vendor/c".to_string(),
            vec![
                wire_advisory("PKSA-3", "*", Some("medium")),
                wire_advisory("PKSA-4", "*", Some("low")),
            ],
        );
        advisories.insert(
            "vendor/d".to_string(),
            vec![wire_advisory("PKSA-5", "*", None)],
        );
        let response = response_with(advisories);
        let packages = vec![
            pkg("vendor/a", "1.0.0"),
            pkg("vendor/b", "1.0.0"),
            pkg("vendor/c", "1.0.0"),
            pkg("vendor/d", "1.0.0"),
        ];

        let result = audit(&packages, &response, SeverityArg::Low);
        assert_eq!(result.summary.critical, 1);
        assert_eq!(result.summary.high, 1);
        assert_eq!(result.summary.medium, 1);
        assert_eq!(result.summary.low, 1);
        assert_eq!(result.summary.unknown, 1);
        assert_eq!(result.vulnerable_packages, 4);
        assert_eq!(result.total_packages, 4);
    }

    #[test]
    fn empty_audit_result() {
        let response = response_with(HashMap::new());
        let packages: Vec<LockedPackage> = vec![];

        let result = audit(&packages, &response, SeverityArg::Low);
        assert_eq!(result.total_packages, 0);
        assert_eq!(result.vulnerable_packages, 0);
        assert!(result.findings.is_empty());
        assert!(result.coverage_unknown.is_empty());
    }

    // --- Serialization ---

    #[test]
    fn severity_serializes_lowercase() {
        let json = serde_json::to_string(&Severity::High).unwrap();
        assert_eq!(json, "\"high\"");

        let json = serde_json::to_string(&Severity::Unknown).unwrap();
        assert_eq!(json, "\"unknown\"");
    }

    #[test]
    fn audit_result_serializes_correctly() {
        let mut advisories = HashMap::new();
        advisories.insert(
            "vendor/pkg".to_string(),
            vec![wire_advisory("PKSA-1", "*", Some("high"))],
        );
        let response = response_with(advisories);
        let packages = vec![pkg("vendor/pkg", "1.0.0")];

        let result = audit(&packages, &response, SeverityArg::Low);
        let json = serde_json::to_string(&result).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["total_packages"], 1);
        assert_eq!(parsed["vulnerable_packages"], 1);
        assert_eq!(parsed["findings"][0]["package"], "vendor/pkg");
        assert_eq!(parsed["findings"][0]["severity"], "high");
        assert_eq!(parsed["summary"]["high"], 1);
        assert!(parsed.get("coverage_unknown").is_none());
    }
}
