use crate::cli::args::Command;
use crate::cli::params::{DownloadParams, GeneratePackagesFileFromLockfileParams, LockParams};
use crate::config::{hash_config_file, load_config};
use crate::download::DownloadAndCheckOptions;
use crate::error::AptPrepError;
use crate::lockfile::Lockfile;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub enum ResolvedCommand {
    Lock(LockParams),
    Download(DownloadParams),
    GeneratePackagesFileFromLockfile(GeneratePackagesFileFromLockfileParams),
}

pub fn resolve_command(command: Command) -> Result<ResolvedCommand, AptPrepError> {
    match command {
        Command::Lock {
            config_path,
            lockfile_path,
            target_architectures,
        } => {
            let app_config = load_config(&config_path)?;

            if app_config.source_repositories.is_empty() {
                return Err(AptPrepError::LockfileValidation {
                    details: "No source repositories defined in config".to_string(),
                });
            }

            let mut resolved_target_architectures = if target_architectures.is_empty() {
                app_config.output.target_architectures.clone()
            } else {
                target_architectures
            };
            resolved_target_architectures.sort();
            resolved_target_architectures.dedup();

            if resolved_target_architectures.is_empty() {
                return Err(AptPrepError::CliArgumentValidation {
                    details: "No target architectures provided. Configure output.target_architectures or pass --target-architecture.".to_string(),
                });
            }

            let config_hash = hash_config_file(Path::new(&config_path))?;

            Ok(ResolvedCommand::Lock(LockParams {
                app_config,
                config_hash,
                lockfile_path: PathBuf::from(lockfile_path),
                target_architectures: resolved_target_architectures,
            }))
        }
        Command::Download {
            config_path,
            lockfile_path,
            output_dir,
            max_concurrency_per_host,
            max_retries,
            download_parallelism,
            checking_parallelism,
        } => {
            for (name, value) in [
                ("max-concurrency-per-host", max_concurrency_per_host),
                ("max-retries", max_retries),
                ("download-parallelism", download_parallelism),
                ("checking-parallelism", checking_parallelism),
            ] {
                if value == 0 {
                    return Err(AptPrepError::CliArgumentValidation {
                        details: format!("{name} must be greater than 0."),
                    });
                }
            }

            let lockfile = Lockfile::load_from_file(Path::new(&lockfile_path))?;

            let resolved_output_dir = match config_path {
                Some(config_path) => {
                    let app_config = load_config(&config_path)?;
                    let config_hash = hash_config_file(Path::new(&config_path))?;
                    if lockfile.config_hash != config_hash {
                        return Err(AptPrepError::LockfileValidation {
                            details: "Configuration hash does not match lockfile. Please regenerate the lockfile with 'aptprep lock'.".to_string(),
                        });
                    }

                    output_dir
                        .map(PathBuf::from)
                        .or_else(|| app_config.output.path.clone())
                        .ok_or_else(|| AptPrepError::CliArgumentValidation {
                            details: "No output directory provided. Configure output.path or pass --output-dir."
                                .to_string(),
                        })?
                }
                None => {
                    output_dir
                        .map(PathBuf::from)
                        .ok_or_else(|| AptPrepError::CliArgumentValidation {
                            details: "No output directory provided. Pass --output-dir or provide --config with output.path."
                                .to_string(),
                        })?
                }
            };

            Ok(ResolvedCommand::Download(DownloadParams {
                lockfile,
                output_dir: resolved_output_dir,
                options: DownloadAndCheckOptions {
                    max_concurrency_per_host,
                    max_retries,
                    download_parallelism,
                    checking_parallelism,
                },
            }))
        }
        Command::GeneratePackagesFileFromLockfile {
            config_path,
            lockfile_path,
            output_path,
        } => {
            let lockfile = Lockfile::load_from_file(Path::new(&lockfile_path))?;

            let output_path = if let Some(output_path) = output_path {
                PathBuf::from(output_path)
            } else if let Some(config_path) = config_path {
                let app_config = load_config(&config_path)?;
                app_config
                    .output
                    .path
                    .map(|path| path.join("Packages"))
                    .ok_or_else(|| AptPrepError::CliArgumentValidation {
                        details: "No output path provided. Pass --output or configure output.path."
                            .to_string(),
                    })?
            } else {
                return Err(AptPrepError::CliArgumentValidation {
                    details:
                        "No output path provided. Pass --output or provide --config with output.path."
                            .to_string(),
                });
            };

            Ok(ResolvedCommand::GeneratePackagesFileFromLockfile(
                GeneratePackagesFileFromLockfileParams {
                    lockfile,
                    output_path,
                },
            ))
        }
    }
}
