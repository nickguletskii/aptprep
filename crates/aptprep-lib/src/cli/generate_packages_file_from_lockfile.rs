use crate::cli::GeneratePackagesFileFromLockfileParams;
use crate::error::AptPrepError;
use crate::output::generate_packages_file_from_lockfile;

pub async fn run_generate_packages_file_from_lockfile(
    params: GeneratePackagesFileFromLockfileParams,
) -> Result<(), AptPrepError> {
    let GeneratePackagesFileFromLockfileParams {
        lockfile,
        output_path,
    } = params;

    tracing::info!("Generating Packages file from lockfile...");

    let final_output_path = generate_packages_file_from_lockfile(&lockfile, &output_path)?;

    tracing::info!(
        "Packages file generated successfully at {:?}",
        final_output_path
    );
    Ok(())
}
