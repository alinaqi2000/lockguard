pub mod audit;
pub mod cli;
pub mod error;
pub mod lock;
pub mod packagist;
pub mod report;

use crate::cli::{Cli, Format};
use crate::error::Result;
use std::process::ExitCode;

pub async fn run(cli: Cli) -> Result<ExitCode> {
    run_with_base_url(cli, packagist::DEFAULT_BASE_URL).await
}

pub async fn run_with_base_url(cli: Cli, base_url: &str) -> Result<ExitCode> {
    eprintln!("Auditing {}...", cli.lock.display());

    let lock_data = lock::read_lock(&cli.lock)?;
    let packages = lock::normalize(&lock_data, &cli.lock)?;

    if packages.is_empty() {
        eprintln!("warning: no packages found in lock file");
    }

    let client = packagist::Client::with_base_url(base_url)?;
    let package_names: Vec<String> = packages.iter().map(|p| p.name.clone()).collect();
    let response = client.fetch_advisories(&package_names).await?;

    let result = audit::audit(&packages, &response, cli.min_severity);

    for pkg in &result.coverage_unknown {
        eprintln!("warning: no advisory data for '{}' — coverage unknown", pkg);
    }

    let stdout = std::io::stdout();
    let mut out = stdout.lock();

    match cli.format {
        Format::Text => report::render_text(&result, &mut out)?,
        Format::Json => report::render_json(&result, &mut out)?,
    }

    if result.findings.is_empty() {
        Ok(ExitCode::SUCCESS)
    } else {
        Ok(ExitCode::from(1))
    }
}
