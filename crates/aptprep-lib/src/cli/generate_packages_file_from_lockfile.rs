use crate::config::load_config;
use crate::error::AptPrepError;
use crate::lockfile::Lockfile;
use crate::output::generate_packages_file_from_lockfile;
use std::path::Path;

pub async fn run_generate_packages_file_from_lockfile(
    config_path: &str,
    lockfile_path: &str,
) -> Result<(), AptPrepError> {
    tracing::info!("Loading configuration from {}", config_path);
    let app_config = load_config(config_path)?;

    tracing::info!("Loading lockfile from {}", lockfile_path);
    let lockfile = Lockfile::load_from_file(Path::new(lockfile_path))?;

    tracing::info!("Generating Packages file from lockfile...");
    let output_path = generate_packages_file_from_lockfile(&lockfile, &app_config.output)?;

    tracing::info!("Packages file generated successfully at {:?}", output_path);
    Ok(())
}
