use crate::config::{hash_config_file, load_config};
use crate::dependency::resolve_dependencies;
use crate::error::AptPrepError;
use crate::lockfile::Lockfile;
use crate::repository::collect_binary_packages;
use std::path::Path;
use tracing;

pub async fn run_lock(config_path: &str, lockfile_path: &str) -> Result<(), AptPrepError> {
    tracing::info!("Loading configuration from {}", config_path);
    let app_config = load_config(config_path)?;

    if app_config.source_repositories.is_empty() {
        return Err(AptPrepError::LockfileValidation {
            details: "No source repositories defined in config".to_string(),
        });
    }

    // Hash the config file for lockfile validation
    let config_hash = hash_config_file(Path::new(config_path))?;

    // Collect binary packages from repositories
    tracing::info!("Collecting binary packages from repositories...");
    let binary_packages = collect_binary_packages(&app_config).await?;

    // Create lockfile
    let mut lockfile = Lockfile::new(config_hash, app_config.packages.clone());

    // Resolve dependencies for each architecture
    tracing::info!("Resolving requirements...");
    for architecture in app_config.output.target_architectures.iter().cloned() {
        tracing::info!("Resolving requirements for {}", architecture);

        let resolved_packages =
            resolve_dependencies(&binary_packages, &app_config.packages, &architecture)?;

        lockfile.add_packages(architecture, &resolved_packages, &binary_packages)?;
    }

    // Save lockfile
    tracing::info!("Saving lockfile to {}", lockfile_path);
    lockfile.save_to_file(Path::new(lockfile_path))?;

    tracing::info!("Lockfile created successfully at {}", lockfile_path);
    Ok(())
}
