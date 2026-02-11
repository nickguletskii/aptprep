use crate::cli::DownloadParams;
use crate::download::{DownloadItem, download_and_check_all};
use crate::error::AptPrepError;
use crate::output::generate_packages_file_from_lockfile;
use debian_packaging::checksum::AnyChecksumType;
use debian_packaging::checksum::AnyContentDigest;
use tracing;

pub async fn run_download(params: DownloadParams) -> Result<(), AptPrepError> {
    let DownloadParams {
        lockfile,
        output_dir,
        options,
    } = params;

    // Create download items from lockfile
    let mut download_items = Vec::new();
    tracing::info!("Processing {} packages", lockfile.packages.len());

    for package in lockfile.packages.values() {
        // Parse the digest
        let checksum_type = match package.digest.algorithm.as_str() {
            "MD5Sum" => AnyChecksumType::Md5,
            "SHA1" => AnyChecksumType::Sha1,
            "SHA256" => AnyChecksumType::Sha256,
            "SHA384" => AnyChecksumType::Sha384,
            "SHA512" => AnyChecksumType::Sha512,
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

        let mut base_url_url = url.clone();
        base_url_url.set_path("");
        base_url_url.set_query(None);
        base_url_url.set_fragment(None);
        let base_url = base_url_url.as_str().trim_end_matches('/').to_string();
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
    download_and_check_all(download_items, output_dir.clone(), options).await?;

    // Generate Packages file from lockfile
    tracing::info!("Generating Packages file...");
    generate_packages_file_from_lockfile(&lockfile, &output_dir.join("Packages"))?;

    tracing::info!("Download completed successfully");
    Ok(())
}
