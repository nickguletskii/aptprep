use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AptPrepError {
    #[error("Configuration error: {0}")]
    Config(#[from] config::ConfigError),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Dependency resolution error: {0}")]
    DependencyResolution(#[from] crate::dependency::DependencyResolutionError),

    #[error("Failed to load lockfile from {path}: {reason}")]
    LockfileLoad { path: PathBuf, reason: String },

    #[error("Failed to save lockfile to {path}: {reason}")]
    LockfileSave { path: PathBuf, reason: String },

    #[error("Lockfile validation failed: {details}")]
    LockfileValidation { details: String },

    #[error("Failed to download package {package} from {url}: {reason}")]
    PackageDownload {
        package: String,
        url: String,
        reason: String,
    },

    #[error("Download error: {message}")]
    Download { message: String },

    #[error("Download directory creation failed at {path}: {reason}")]
    DownloadDirectoryCreation { path: PathBuf, reason: String },

    #[error("Repository access failed for {repository}: {reason}")]
    RepositoryAccess { repository: String, reason: String },

    #[error("Package verification failed for {package}: expected {expected}, got {actual}")]
    PackageVerification {
        package: String,
        expected: String,
        actual: String,
    },

    #[error("Package validation failed for {package}: {details}")]
    PackageValidation { package: String, details: String },

    #[error("Failed to hash configuration file {path}: {reason}")]
    ConfigFileHash { path: PathBuf, reason: String },

    #[error("JSON serialization/deserialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("HTTP request error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Debian packaging error: {0}")]
    DebianPackaging(#[from] debian_packaging::error::DebianError),

    #[error("Unexpected error: {0}")]
    Unexpected(#[from] eyre::Report),
}
