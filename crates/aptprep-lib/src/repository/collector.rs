use super::types::{BinaryPackage, SourceInfo};
use crate::config::{Config, DistributionDef};
use crate::error::AptPrepError;
use debian_packaging::repository::reader_from_str;
use std::collections::HashMap;
use std::sync::Arc;
use tracing;

pub async fn collect_binary_packages(
    app_config: &Config,
) -> Result<HashMap<String, Vec<BinaryPackage>>, AptPrepError> {
    let mut binary_packages_by_arch: HashMap<String, Vec<BinaryPackage>> = HashMap::new();

    for source_repository in app_config.source_repositories.iter() {
        let reader = reader_from_str(&source_repository.source_url).map_err(|e| {
            AptPrepError::RepositoryAccess {
                repository: source_repository.source_url.clone(),
                reason: format!("Couldn't read repository: {}", e),
            }
        })?;
        tracing::info!(
            "Processing source repository: {}",
            source_repository.source_url
        );

        for distribution in source_repository.distributions.iter() {
            let (release, url) = match distribution {
                DistributionDef::Simple(name) => (
                    reader.release_reader(name).await.map_err(|e| {
                        AptPrepError::RepositoryAccess {
                            repository: source_repository.source_url.clone(),
                            reason: format!("Couldn't fetch release: {}", e),
                        }
                    })?,
                    reader.url().expect("Release has no URL"),
                ),
                DistributionDef::Advanced { distribution_path } => (
                    reader
                        .release_reader_with_distribution_path(distribution_path)
                        .await
                        .map_err(|e| AptPrepError::RepositoryAccess {
                            repository: source_repository.source_url.clone(),
                            reason: format!("Couldn't fetch release: {}", e),
                        })?,
                    reader
                        .url()
                        .expect("Release has no URL")
                        .join(distribution_path)
                        .expect("Invalid URL"),
                ),
            };

            let package_indices = release
                .packages_indices_entries_preferred_compression()
                .map_err(|e| AptPrepError::RepositoryAccess {
                    repository: source_repository.source_url.clone(),
                    reason: format!("Couldn't read package indices list: {}", e),
                })?;

            for package_entry in package_indices.iter() {
                if package_entry.architecture != "all"
                    && !source_repository
                        .architectures
                        .iter()
                        .any(|architecture| architecture.as_str() == package_entry.architecture)
                {
                    continue;
                }

                let packages_list = release.resolve_packages_from_entry(package_entry).await?;
                for binary_package in packages_list.iter() {
                    let Ok(package_name) = binary_package.package() else {
                        tracing::warn!("Skipping package, no package name specified");
                        continue;
                    };
                    let Ok(architecture) = binary_package.architecture() else {
                        tracing::warn!(
                            package = package_name,
                            "Skipping package, no architecture specified"
                        );
                        continue;
                    };

                    binary_packages_by_arch
                        .entry(architecture.to_string())
                        .or_default()
                        .push(BinaryPackage {
                            source_info: Arc::new(SourceInfo { url: url.clone() }),
                            control_file: Arc::new(binary_package.clone()),
                        });
                }
            }
        }
    }
    Ok(binary_packages_by_arch)
}
