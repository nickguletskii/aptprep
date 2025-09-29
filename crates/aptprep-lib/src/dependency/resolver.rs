use super::provider::AptDependencyProvider;
use super::types::{AptDependencyGraphElement, AptVersion, RequestedPackages};
use crate::repository::types::{BinaryPackage, iterate_all_relevant_packages};
use debian_packaging::binary_package_control::BinaryPackageControlFile;
use debian_packaging::error::DebianError;
use debian_packaging::package_version::PackageVersion;
use eyre::WrapErr;
use pubgrub::{DefaultStringReporter, PubGrubError, Reporter, resolve};
use std::collections::{BTreeSet, HashMap};
use std::sync::Arc;
use thiserror::Error;
use tracing;

#[derive(Error, Debug)]
pub enum DependencyResolutionError {
    #[error("Debian packaging error: {0}")]
    DebianError(#[from] DebianError),
    #[error("PubGrub error: {0}")]
    PubGrubError(String),
    #[error("Configuration error: {0}")]
    ConfigError(String),
    #[error("Unexpected error: {0}")]
    Unexpected(#[from] eyre::Report),
}

pub fn resolve_dependencies(
    binary_packages: &HashMap<String, Vec<BinaryPackage>>,
    required_packages: &[Arc<str>],
    architecture: &str,
) -> Result<BTreeSet<Arc<BinaryPackageControlFile<'static>>>, DependencyResolutionError> {
    tracing::info!("Loading packages for {}", &architecture);
    let dependency_provider = AptDependencyProvider::new(
        iterate_all_relevant_packages(binary_packages, &architecture.to_string())
            .map(|v| v.control_file.clone()),
        architecture,
    )
    .wrap_err("Failed to prepare for pubgrub dependency resolution")?;

    let resolved = match resolve(
        &dependency_provider,
        AptDependencyGraphElement::RequestedPackages(Arc::new(RequestedPackages::from(
            required_packages.iter().cloned(),
        ))),
        AptVersion::from(PackageVersion::parse("1.0.0").unwrap()),
    ) {
        Ok(solution) => solution,
        Err(PubGrubError::NoSolution(mut derivation_tree)) => {
            derivation_tree.collapse_no_versions();
            tracing::error!(
                "No solution: {}",
                DefaultStringReporter::report(&derivation_tree)
            );
            return Err(DependencyResolutionError::PubGrubError(
                "No solution".to_string(),
            ));
        }
        Err(PubGrubError::ErrorChoosingVersion { package, source }) => {
            tracing::error!("Error choosing package version: {} {:?}", package, source);
            return Err(DependencyResolutionError::PubGrubError(
                "Error choosing package version".to_string(),
            ));
        }
        Err(err) => {
            tracing::error!("Error: {}", err);
            return Err(DependencyResolutionError::PubGrubError(
                "Failed to resolve dependencies".to_string(),
            ));
        }
    };

    // tracing::info!("Resolved: {:?}", resolved);
    let mut collected_packages = BTreeSet::new();

    for (package, version) in resolved {
        match package {
            AptDependencyGraphElement::AptPackage(package_name) => {
                if let Some(package) = dependency_provider.get_control(&package_name, &version) {
                    collected_packages.insert(package.clone());
                } else {
                    tracing::warn!(
                        "Package {} with version {} not found",
                        package_name,
                        version
                    );
                }
            }
            AptDependencyGraphElement::DummyPackage(_)
            | AptDependencyGraphElement::RequestedPackages(_) => {
                // Skip dummy packages
            }
        }
    }

    Ok(collected_packages)
}
