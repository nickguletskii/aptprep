use crate::cli::LockParams;
use crate::dependency::resolve_dependencies;
use crate::error::AptPrepError;
use crate::lockfile::Lockfile;
use crate::repository::collect_binary_packages;
use tracing;

pub async fn run_lock(params: LockParams) -> Result<(), AptPrepError> {
    let LockParams {
        app_config,
        config_hash,
        lockfile_path,
        target_architectures,
    } = params;

    // Collect binary packages from repositories
    tracing::info!("Collecting binary packages from repositories...");
    let binary_packages = collect_binary_packages(&app_config).await?;

    // Create lockfile
    let mut lockfile = Lockfile::new(config_hash, app_config.packages.clone());

    // Resolve dependencies for each architecture
    tracing::info!("Resolving requirements...");
    for architecture in target_architectures {
        tracing::info!("Resolving requirements for {}", architecture);

        let resolved_packages =
            resolve_dependencies(&binary_packages, &app_config.packages, &architecture)?;

        lockfile.add_packages(architecture, &resolved_packages, &binary_packages)?;
    }

    // Save lockfile
    tracing::info!("Saving lockfile to {}", lockfile_path.display());
    lockfile.save_to_file(&lockfile_path)?;

    tracing::info!(
        "Lockfile created successfully at {}",
        lockfile_path.display()
    );
    Ok(())
}
