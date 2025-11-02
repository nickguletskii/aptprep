use super::types::DownloadItem;
use crate::verification::content_digest_hasher::ContentDigestVerifier;
use debian_packaging::checksum::AnyContentDigest;
use eyre::{Result, WrapErr, eyre};
use futures::stream::{FuturesUnordered, StreamExt};
use md5::Md5;
use opendal::Operator;
use opendal::layers::{ConcurrentLimitLayer, RetryLayer};
use opendal::services::Http;
use sha1::Sha1;
use sha2::{Digest as Sha2Digest, Sha256, Sha384, Sha512};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, warn};

fn build_http_operator(
    base_url: &str,
    max_in_flight: usize,
    max_retries: usize,
) -> Result<Operator> {
    // Build an OpenDAL HTTP service for the given repository base URL.
    // We set the endpoint to the base URL and fetch relative paths below it.
    let builder = Http::default().endpoint(base_url);

    let op = Operator::new(builder)?
        .layer(RetryLayer::new().with_max_times(max_retries))
        .layer(ConcurrentLimitLayer::new(max_in_flight))
        .finish();
    Ok(op)
}

pub async fn download_and_check_all(
    items: Vec<DownloadItem>,
    output_dir: impl AsRef<std::path::Path>,
    // Tuning knobs; feel free to wire from config/CLI if needed
    max_concurrency_per_host: usize,
    max_retries: usize,
    download_parallelism: usize,
    checking_parallelism: usize,
) -> Result<()> {
    // Build per-base operator so multiple items from the same repo reuse the same HTTP client.
    let mut per_base: HashMap<String, Operator> = HashMap::new();
    tracing::info!("Creating operators...");
    for it in &items {
        let key = it.base_url.as_str().to_string();
        if let std::collections::hash_map::Entry::Vacant(e) = per_base.entry(key) {
            let op = build_http_operator(&it.base_url, max_concurrency_per_host, max_retries)?;
            e.insert(op);
        }
    }
    tracing::info!("Starting...");

    // Use a bounded unordered stream for parallel downloads across bases.
    // Throttling per-host is handled by ConcurrentLimitLayer; this controls overall parallelism.
    let download_semaphore = Arc::new(tokio::sync::Semaphore::new(download_parallelism));
    let checking_semaphore = Arc::new(tokio::sync::Semaphore::new(checking_parallelism));

    let mut futs = FuturesUnordered::new();
    for it in items {
        let key = it.base_url.as_str().to_string();
        let op = per_base
            .get(&key)
            .expect("operator must be present")
            .clone();
        let output_dir = output_dir.as_ref().to_path_buf();
        let download_semaphore = download_semaphore.clone();
        let checking_semaphore = checking_semaphore.clone();
        futs.push(async move {
            let permit = checking_semaphore.acquire_owned().await?;
            let rel = it.rel_path.clone();

            // Determine output path
            let output_path = match &it.output_path {
                Some(custom_path) => output_dir.join(custom_path),
                None => output_dir.join(&rel),
            };
            tracing::trace!(base = %key, path = %rel, output = %output_path.display(), "Checking");

            // Ensure parent directory exists
            if let Some(parent) = output_path.parent() {
                std::fs::create_dir_all(parent)
                    .wrap_err_with(|| format!("Failed to create directory: {}", parent.display()))?;
            }

            // Check if file already exists
            if output_path.exists() {
                // Verify the digest of the existing file using streaming to avoid loading the entire file into memory
                let mut sha1_hasher = Sha1::new();
                let mut sha256_hasher = Sha256::new();
                let mut sha384_hasher = Sha384::new();
                let mut sha512_hasher = Sha512::new();
                let mut md5_hasher = Md5::new();

                let file = tokio::fs::File::open(&output_path)
                    .await
                    .wrap_err_with(|| format!("Failed to open existing file: {}", output_path.display()))?;
                let mut reader = tokio::io::BufReader::new(file);
                let mut buffer = vec![0u8; 65536]; // 64KB buffer for reading chunks

                loop {
                    let bytes_read = tokio::io::AsyncReadExt::read(&mut reader, &mut buffer)
                        .await
                        .wrap_err_with(|| format!("Failed to read from existing file: {}", output_path.display()))?;

                    if bytes_read == 0 {
                        break; // End of file
                    }

                    // Only update the hasher for the expected digest type
                    tokio::task::block_in_place(|| match &it.digest {
                        AnyContentDigest::Sha1(_) => sha1_hasher.update(&buffer[..bytes_read]),
                        AnyContentDigest::Sha256(_) => sha256_hasher.update(&buffer[..bytes_read]),
                        AnyContentDigest::Sha384(_) => sha384_hasher.update(&buffer[..bytes_read]),
                        AnyContentDigest::Sha512(_) => sha512_hasher.update(&buffer[..bytes_read]),
                        AnyContentDigest::Md5(_) => md5_hasher.update(&buffer[..bytes_read]),
                    });
                }

                let existing_digest_valid = match &it.digest {
                    AnyContentDigest::Sha1(expected) => {
                        let calculated = sha1_hasher.finalize();
                        calculated.as_slice() == expected.as_slice()
                    },
                    AnyContentDigest::Sha256(expected) => {
                        let calculated = sha256_hasher.finalize();
                        calculated.as_slice() == expected.as_slice()
                    },
                    AnyContentDigest::Sha384(expected) => {
                        let calculated = sha384_hasher.finalize();
                        calculated.as_slice() == expected.as_slice()
                    },
                    AnyContentDigest::Sha512(expected) => {
                        let calculated = sha512_hasher.finalize();
                        calculated.as_slice() == expected.as_slice()
                    },
                    AnyContentDigest::Md5(expected) => {
                        let calculated = md5_hasher.finalize();
                        calculated.as_slice() == expected.as_slice()
                    },
                };

                if existing_digest_valid {
                    tracing::debug!(base = %key, path = %rel, output = %output_path.display(), "File exists with matching digest, skipping download");
                    return Ok(());
                } else {
                    // Delete the file with mismatched digest
                    tracing::info!(base = %key, path = %rel, output = %output_path.display(), "File exists with incorrect digest, deleting");
                    tokio::fs::remove_file(&output_path)
                        .await
                        .wrap_err_with(|| format!("Failed to delete file with incorrect digest: {}", output_path.display()))?;
                }
            }
            drop(permit);

            let _permit = download_semaphore.acquire_owned().await?;
            tracing::info!(base = %key, path = %rel, output = %output_path.display(), expected_digest = it.digest.digest_hex(), "Downloading");

            let mut hasher = ContentDigestVerifier::new(it.digest.clone());

            // Stream the file to disk while calculating hash
            let mut reader = op.reader(&rel)
                .await
                .wrap_err_with(|| format!("Failed to create reader for {}{}", key, rel))?
                .into_stream(..)
                .await
                .wrap_err_with(|| format!("Failed to create reader for {}{}", key, rel))?;

            let file = tokio::fs::File::create(&output_path)
                .await
                .wrap_err_with(|| format!("Failed to create output file: {}", output_path.display()))?;
            let mut writer = tokio::io::BufWriter::new(file);


            loop {
                let Some(reader_res) = reader.next().await else {
                    break;
                };
                let buffer = reader_res.wrap_err_with(|| format!("Failed to read from {}{}", key, rel))?.to_bytes();

                // Update the appropriate hasher based on digest type
                tokio::task::block_in_place(|| hasher.update(&buffer));

                tokio::io::AsyncWriteExt::write_all(&mut writer, &buffer)
                    .await
                    .wrap_err_with(|| format!("Failed to write to {}", output_path.display()))?;

            }

            // Finalize the write
            tokio::io::AsyncWriteExt::flush(&mut writer)
                .await
                .wrap_err_with(|| format!("Failed to flush {}", output_path.display()))?;

            // Verify the hash
            hasher.verify().wrap_err_with(|| format!("Failed to verify {}", output_path.display()))?;
            info!(base = %key, path = %rel, output = %output_path.display(), "Downloaded and verified");
            Ok::<(), eyre::Report>(())
        });
    }

    tracing::info!("Waiting for downloads to finish...");

    let mut failures = Vec::new();
    while let Some(res) = futs.next().await {
        if let Err(err) = res {
            warn!("Download failed: {:#}", err);
            failures.push(err);
        }
    }

    if failures.is_empty() {
        Ok(())
    } else {
        Err(eyre!("{} downloads failed", failures.len()))
    }
}
