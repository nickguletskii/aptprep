use aptprep_lib::config::{Config, DistributionDef, OutputConfig, SourceRepository};
use eyre::Result;
use std::path::Path;
use std::sync::Arc;
use tempfile::TempDir;

pub fn create_test_config() -> Config {
    Config {
        packages: vec![Arc::from("curl"), Arc::from("vim")],
        source_repositories: vec![Arc::new(SourceRepository {
            source_url: "https://snapshot.ubuntu.com/ubuntu/20250910T140000Z".to_string(),
            distributions: vec![DistributionDef::Simple("noble".to_string())],
            architectures: vec!["amd64".to_string()],
        })],
        output: OutputConfig {
            target_architectures: vec!["amd64".to_string()],
            path: "/tmp/test_output".into(),
        },
    }
}

pub fn setup_test_environment() -> Result<TempDir> {
    let temp_dir = tempfile::tempdir()?;

    let config = create_test_config();
    let config_path = temp_dir.path().join("config.json");
    std::fs::write(&config_path, serde_json::to_string_pretty(&config)?)?;

    Ok(temp_dir)
}

pub async fn wait_for_file_creation(path: &Path, timeout_secs: u64) -> bool {
    let start = std::time::Instant::now();
    while start.elapsed().as_secs() < timeout_secs {
        if path.exists() {
            return true;
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
    false
}
