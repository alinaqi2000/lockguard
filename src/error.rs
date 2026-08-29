use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to read lock file {path}: {source}")]
    LockRead {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("invalid lock file {path}: {message}")]
    LockParse { path: PathBuf, message: String },

    #[error("lock file {path}: conflicting versions for '{package}': {first} vs {second}")]
    LockConflict {
        path: PathBuf,
        package: String,
        first: String,
        second: String,
    },

    #[error("lock file {path}: package with empty name or version")]
    LockEmptyIdentity { path: PathBuf },

    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Packagist API returned status {status} for {url}")]
    HttpStatus { status: u16, url: String },

    #[error("failed to decode Packagist response: {0}")]
    ResponseDecode(#[from] serde_json::Error),

    #[error("failed to build HTTP client: {0}")]
    ClientBuild(String),

    #[error("failed to write report: {0}")]
    ReportWrite(String),
}

pub type Result<T> = std::result::Result<T, Error>;
