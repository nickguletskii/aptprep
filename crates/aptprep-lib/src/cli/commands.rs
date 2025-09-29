use crate::config::load_config;
use crate::dependency::resolve_dependencies;
use crate::download::download_and_check_all;
use crate::output::generate_packages_file;
use crate::repository::collect_binary_packages;
use eyre::Report;
use std::collections::{BTreeSet, HashMap};
use tracing;

#[allow(dead_code)]
pub async fn run(config_path: &str) -> Result<(), Report> {
    tracing::info!("Loading configuration from {}", config_path);
    let app_config = load_config(config_path)?;

    if app_config.source_repositories.is_empty() {
        return Err(eyre::eyre!("No source repositories defined in config"));
    }

    let binary_packages = collect_binary_packages(&app_config).await?;

    let binary_packages_by_control_file = binary_packages
        .values()
        .flatten()
        .map(|v| (v.control_file.clone(), v))
        .collect::<HashMap<_, _>>();

    tracing::info!("Resolving requirements...");
    let mut collected_packages = BTreeSet::new();

    for architecture in app_config.output.target_architectures.iter() {
        tracing::info!("Resolving requirements for {}", architecture);

        let resolved_packages =
            resolve_dependencies(&binary_packages, &app_config.packages, architecture)?;
        collected_packages.extend(resolved_packages);
    }

    let (fetches, _packages_file_path) = generate_packages_file(
        &collected_packages,
        &binary_packages_by_control_file,
        &app_config.output,
    )?;

    tracing::info!("Downloading packages...");
    download_and_check_all(fetches, app_config.output.path, 8, 5, 16, 128).await?;

    Ok(())
}
