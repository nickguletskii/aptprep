use aptprep_e2e_tests::{create_test_config, setup_test_environment, wait_for_file_creation};
use aptprep_lib::cli::{Command, ResolvedCommand, resolve_command, run_lock};
use aptprep_lib::lockfile::Lockfile;

fn build_lock_params(config_path: &str, lockfile_path: &str) -> aptprep_lib::cli::LockParams {
    let command = Command::Lock {
        config_path: config_path.to_string(),
        lockfile_path: lockfile_path.to_string(),
        target_architectures: vec![],
    };
    match resolve_command(command).expect("Failed to resolve lock command") {
        ResolvedCommand::Lock(params) => params,
        _ => unreachable!("Resolved command type mismatch"),
    }
}

#[tokio::test]
async fn test_lockfile_generation_end_to_end() {
    init_tracing();

    let temp_dir = setup_test_environment().expect("Failed to setup test environment");

    let config_path = temp_dir.path().join("config.json");
    let lockfile_path = temp_dir.path().join("aptprep.lock");

    let params = build_lock_params(
        config_path.to_str().unwrap(),
        lockfile_path.to_str().unwrap(),
    );
    let result = run_lock(params).await;

    assert!(
        result.is_ok(),
        "Lockfile generation should succeed: {:?}",
        result
    );

    assert!(
        wait_for_file_creation(&lockfile_path, 10).await,
        "Lockfile should be created within 10 seconds"
    );

    let lockfile = Lockfile::load_from_file(&lockfile_path)
        .expect("Should be able to load generated lockfile");

    assert!(
        !lockfile.packages.is_empty(),
        "Lockfile should contain packages"
    );

    for (package_key, package) in &lockfile.packages {
        let package_name = package.package_name().expect("Should have package name");
        let package_version = package
            .package_version()
            .expect("Should have package version");

        assert!(!package_key.is_empty(), "Package key should not be empty");
        assert!(!package_name.is_empty(), "Package name should not be empty");
        assert!(
            !package_version.is_empty(),
            "Package version should not be empty"
        );
        assert!(
            !package.architecture.is_empty(),
            "Package architecture should not be empty"
        );
        assert!(
            !package.download_url.is_empty(),
            "Package download URL should not be empty"
        );
        assert!(package.size > 0, "Package size should be greater than 0");
        assert!(
            !package.digest.value.is_empty(),
            "Package digest should not be empty"
        );
    }

    assert!(
        !lockfile.package_groups.is_empty(),
        "Lockfile should contain package groups"
    );

    let config = create_test_config();
    assert_eq!(
        lockfile.required_packages, config.packages,
        "Lockfile should contain the same required packages as config"
    );
}

#[tokio::test]
async fn test_lockfile_contains_expected_packages() {
    init_tracing();

    let temp_dir = setup_test_environment().expect("Failed to setup test environment");

    let config_path = temp_dir.path().join("config.json");
    let lockfile_path = temp_dir.path().join("aptprep.lock");

    let params = build_lock_params(
        config_path.to_str().unwrap(),
        lockfile_path.to_str().unwrap(),
    );
    run_lock(params)
        .await
        .expect("Lockfile generation should succeed");

    let lockfile = Lockfile::load_from_file(&lockfile_path)
        .expect("Should be able to load generated lockfile");

    let package_names: Vec<String> = lockfile
        .packages
        .values()
        .filter_map(|p| p.package_name().ok())
        .collect();

    assert!(
        package_names.contains(&"curl".to_string()),
        "Should contain curl package, found: {:?}",
        package_names
    );

    assert!(
        package_names.contains(&"vim".to_string()),
        "Should contain vim package, found: {:?}",
        package_names
    );
}

#[tokio::test]
async fn test_lockfile_reproducibility() {
    init_tracing();

    let temp_dir = setup_test_environment().expect("Failed to setup test environment");

    let config_path = temp_dir.path().join("config.json");
    let lockfile_path1 = temp_dir.path().join("aptprep1.lock");
    let lockfile_path2 = temp_dir.path().join("aptprep2.lock");

    let params1 = build_lock_params(
        config_path.to_str().unwrap(),
        lockfile_path1.to_str().unwrap(),
    );
    run_lock(params1)
        .await
        .expect("First lockfile generation should succeed");

    let params2 = build_lock_params(
        config_path.to_str().unwrap(),
        lockfile_path2.to_str().unwrap(),
    );
    run_lock(params2)
        .await
        .expect("Second lockfile generation should succeed");

    let lockfile1 =
        Lockfile::load_from_file(&lockfile_path1).expect("Should be able to load first lockfile");
    let lockfile2 =
        Lockfile::load_from_file(&lockfile_path2).expect("Should be able to load second lockfile");

    assert_eq!(
        lockfile1.packages, lockfile2.packages,
        "Lockfiles should be identical when generated from same config"
    );
    assert_eq!(
        lockfile1.config_hash, lockfile2.config_hash,
        "Config hashes should be identical"
    );
}

fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter("aptprep=debug,aptprep_e2e_tests=debug")
        .with_test_writer()
        .try_init()
        .ok();
}
