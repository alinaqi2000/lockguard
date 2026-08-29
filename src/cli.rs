use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "lockguard",
    version,
    about = "Audit composer.lock for known security vulnerabilities"
)]
pub struct Cli {
    /// Path to composer.lock
    #[arg(long, default_value = "composer.lock")]
    pub lock: PathBuf,

    /// Output format
    #[arg(long, value_enum, default_value_t = Format::Text)]
    pub format: Format,

    /// Minimum severity to report
    #[arg(long, value_enum, default_value_t = SeverityArg::Low)]
    pub min_severity: SeverityArg,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Format {
    Text,
    Json,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum SeverityArg {
    Low,
    Medium,
    High,
    Critical,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn defaults() {
        let cli = Cli::parse_from(["lockguard"]);
        assert_eq!(cli.lock, PathBuf::from("composer.lock"));
        assert_eq!(cli.format, Format::Text);
        assert_eq!(cli.min_severity, SeverityArg::Low);
    }

    #[test]
    fn custom_lock_path() {
        let cli = Cli::parse_from(["lockguard", "--lock", "path/to/composer.lock"]);
        assert_eq!(cli.lock, PathBuf::from("path/to/composer.lock"));
    }

    #[test]
    fn json_format() {
        let cli = Cli::parse_from(["lockguard", "--format", "json"]);
        assert_eq!(cli.format, Format::Json);
    }

    #[test]
    fn text_format_explicit() {
        let cli = Cli::parse_from(["lockguard", "--format", "text"]);
        assert_eq!(cli.format, Format::Text);
    }

    #[test]
    fn min_severity_medium() {
        let cli = Cli::parse_from(["lockguard", "--min-severity", "medium"]);
        assert_eq!(cli.min_severity, SeverityArg::Medium);
    }

    #[test]
    fn min_severity_high() {
        let cli = Cli::parse_from(["lockguard", "--min-severity", "high"]);
        assert_eq!(cli.min_severity, SeverityArg::High);
    }

    #[test]
    fn min_severity_critical() {
        let cli = Cli::parse_from(["lockguard", "--min-severity", "critical"]);
        assert_eq!(cli.min_severity, SeverityArg::Critical);
    }

    #[test]
    fn all_options() {
        let cli = Cli::parse_from([
            "lockguard",
            "--lock",
            "my.lock",
            "--format",
            "json",
            "--min-severity",
            "high",
        ]);
        assert_eq!(cli.lock, PathBuf::from("my.lock"));
        assert_eq!(cli.format, Format::Json);
        assert_eq!(cli.min_severity, SeverityArg::High);
    }

    #[test]
    fn invalid_format_rejected() {
        assert!(Cli::try_parse_from(["lockguard", "--format", "xml"]).is_err());
    }

    #[test]
    fn invalid_severity_rejected() {
        assert!(Cli::try_parse_from(["lockguard", "--min-severity", "extreme"]).is_err());
    }

    #[test]
    fn severity_ordering() {
        assert!(SeverityArg::Low < SeverityArg::Medium);
        assert!(SeverityArg::Medium < SeverityArg::High);
        assert!(SeverityArg::High < SeverityArg::Critical);
    }
}
