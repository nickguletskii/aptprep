mod loader;
mod model;

pub use loader::load_config;
pub use model::{Config, DistributionDef, OutputConfig, SourceRepository};

use sha2::{Digest, Sha256};
use std::path::Path;

pub fn hash_config_file(config_path: &Path) -> Result<String, crate::error::AptPrepError> {
    let content =
        std::fs::read(config_path).map_err(|e| crate::error::AptPrepError::ConfigFileHash {
            path: config_path.to_path_buf(),
            reason: e.to_string(),
        })?;
    let mut hasher = Sha256::new();
    hasher.update(&content);
    Ok(format!("{:x}", hasher.finalize()))
}
