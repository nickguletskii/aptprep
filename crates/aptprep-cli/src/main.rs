use aptprep_lib::cli::{
    ResolvedCommand, parse_args, resolve_command, run_download,
    run_generate_packages_file_from_lockfile, run_lock,
};
use aptprep_lib::error::AptPrepError;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), AptPrepError> {
    color_eyre::install()?;

    let args = parse_args();
    let command = resolve_command(args.command)?;

    match command {
        ResolvedCommand::Lock(params) => run_lock(params).await?,
        ResolvedCommand::Download(params) => run_download(params).await?,
        ResolvedCommand::GeneratePackagesFileFromLockfile(params) => {
            run_generate_packages_file_from_lockfile(params).await?
        }
    }

    Ok(())
}
