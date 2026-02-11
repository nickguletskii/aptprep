use crate::config::Config;
use crate::download::DownloadAndCheckOptions;
use crate::lockfile::Lockfile;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct LockParams {
    pub app_config: Config,
    pub config_hash: String,
    pub lockfile_path: PathBuf,
    pub target_architectures: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct DownloadParams {
    pub lockfile: Lockfile,
    pub output_dir: PathBuf,
    pub options: DownloadAndCheckOptions,
}

#[derive(Debug, Clone)]
pub struct GeneratePackagesFileFromLockfileParams {
    pub lockfile: Lockfile,
    pub output_path: PathBuf,
}
