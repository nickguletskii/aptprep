use crate::config::{hash_config_file, load_config};
use crate::download::{DownloadItem, download_and_check_all};
use crate::error::AptPrepError;
use crate::lockfile::Lockfile;
use crate::output::generate_packages_file_from_lockfile;
use debian_packaging::checksum::AnyChecksumType;
use debian_packaging::checksum::AnyContentDigest;
use std::path::Path;
use tracing;

pub async fn run_download(config_path: &str, lockfile_path: &str) -> Result<(), AptPrepError> {
    tracing::info!("Loading configuration from {}", config_path);
    let app_config = load_config(config_path)?;

    tracing::info!("Loading lockfile from {}", lockfile_path);
    let lockfile = Lockfile::load_from_file(Path::new(lockfile_path))?;

    // Verify config hash matches
    let config_hash = hash_config_file(Path::new(config_path))?;
    if lockfile.config_hash != config_hash {
        tracing::warn!(
            "Configuration file has changed since lockfile was created. \
             Consider regenerating the lockfile with 'aptprep lock'"
        );
    }

    // Verify required packages match
    if lockfile.required_packages != app_config.packages {
        return Err(AptPrepError::LockfileValidation {
            details: "Required packages in lockfile don't match configuration. \
             Please regenerate the lockfile with 'aptprep lock'"
                .to_string(),
        });
    }

    // Create download items from lockfile
    let mut download_items = Vec::new();
    tracing::info!("Processing {} packages", lockfile.packages.len());

    for package in lockfile.packages.values() {
        // Parse the digest
        let checksum_type = match package.digest.algorithm.as_str() {
            "MD5Sum" => AnyChecksumType::Md5,
            "SHA1" => AnyChecksumType::Sha1,
            "SHA256" => AnyChecksumType::Sha256,
            _ => {
                return Err(AptPrepError::PackageVerification {
                    package: "unknown".to_string(),
                    expected: "supported digest algorithm".to_string(),
                    actual: package.digest.algorithm.clone(),
                });
            }
        };

        let digest = AnyContentDigest::from_hex_digest(checksum_type, &package.digest.value)?;

        // Extract filename from download URL
        let filename =
            package
                .download_url
                .split('/')
                .next_back()
                .ok_or_else(|| AptPrepError::Download {
                    message: format!("Invalid download URL: {}", package.download_url),
                })?;

        // Parse the download URL to separate base and relative path
        let url =
            reqwest::Url::parse(&package.download_url).map_err(|e| AptPrepError::Download {
                message: format!("Invalid download URL {}: {}", package.download_url, e),
            })?;

        let base_url = format!("{}://{}", url.scheme(), url.host_str().unwrap_or(""));
        let rel_path = url.path().to_string();

        download_items.push(DownloadItem {
            base_url,
            rel_path,
            size: Some(package.size),
            digest,
            output_path: Some(filename.to_string()),
        });
    }

    tracing::info!("Downloading {} packages...", download_items.len());
    download_and_check_all(
        download_items,
        app_config.output.path.clone(),
        8,
        5,
        16,
        128,
    )
    .await?;

    // Generate Packages file from lockfile
    tracing::info!("Generating Packages file...");
    generate_packages_file_from_lockfile(&lockfile, &app_config.output)?;

    tracing::info!("Download completed successfully");
    Ok(())
}
