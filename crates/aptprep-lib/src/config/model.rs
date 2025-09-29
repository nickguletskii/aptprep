use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SourceRepository {
    pub source_url: String,
    pub architectures: Vec<String>,
    pub distributions: Vec<DistributionDef>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, untagged)]
pub enum DistributionDef {
    Simple(String),
    Advanced { distribution_path: String },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub source_repositories: Vec<Arc<SourceRepository>>,
    pub packages: Vec<Arc<str>>,
    pub output: OutputConfig,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OutputConfig {
    pub path: PathBuf,
    pub target_architectures: Vec<String>,
}
