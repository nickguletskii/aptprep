use aptprep_lib::cli::{
    Command, parse_args, run_download, run_generate_packages_file_from_lockfile, run_lock,
};
use aptprep_lib::error::AptPrepError;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), AptPrepError> {
    color_eyre::install()?;

    let args = parse_args();

    match args.command {
        Command::Lock {
            config_path,
            lockfile_path,
        } => {
            run_lock(&config_path, &lockfile_path).await?;
        }
        Command::Download {
            config_path,
            lockfile_path,
        } => {
            run_download(&config_path, &lockfile_path).await?;
        }
        Command::GeneratePackagesFileFromLockfile {
            config_path,
            lockfile_path,
        } => {
            run_generate_packages_file_from_lockfile(&config_path, &lockfile_path).await?;
        }
    }

    Ok(())
}
