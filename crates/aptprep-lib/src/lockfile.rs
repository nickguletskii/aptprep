use debian_packaging::binary_package_control::BinaryPackageControlFile;
use debian_packaging::io::ContentDigest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LockfilePackageEntry {
    /// Package name
    pub name: String,
    /// Package version
    pub version: String,
    /// Target architecture
    pub architecture: String,
    /// Complete download URL
    pub download_url: String,
    /// File size in bytes
    pub size: u64,
    /// Content digest for verification
    pub digest: LockfileDigest,
    /// Dependencies as package keys
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LockfileDigest {
    pub algorithm: String,
    pub value: String,
}

impl From<&ContentDigest> for LockfileDigest {
    fn from(digest: &ContentDigest) -> Self {
        match digest {
            ContentDigest::Md5(bytes) => Self {
                algorithm: "MD5Sum".to_string(),
                value: hex::encode(bytes),
            },
            ContentDigest::Sha1(bytes) => Self {
                algorithm: "SHA1".to_string(),
                value: hex::encode(bytes),
            },
            ContentDigest::Sha256(bytes) => Self {
                algorithm: "SHA256".to_string(),
                value: hex::encode(bytes),
            },
        }
    }
}

impl LockfilePackageEntry {
    /// Get package name
    pub fn package_name(&self) -> Result<String, crate::error::AptPrepError> {
        Ok(self.name.clone())
    }

    /// Get package version
    pub fn package_version(&self) -> Result<String, crate::error::AptPrepError> {
        Ok(self.version.clone())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Lockfile {
    /// Version of the lockfile format
    pub version: u32,
    /// Hash of the configuration used to generate this lockfile
    pub config_hash: String,
    /// Required packages from config
    pub required_packages: Vec<Arc<str>>,
    /// Resolved packages by unique key
    pub packages: HashMap<String, LockfilePackageEntry>,
    /// Package groups by name for multi-arch support
    pub package_groups: HashMap<String, Vec<String>>,
}

fn sanitize_package_key_component(component: &str) -> String {
    component
        .chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' => c,
            _ => '_',
        })
        .collect()
}

fn generate_package_key(architecture: &str, name: &str, version: &str) -> String {
    format!(
        "{}_{}_{}",
        sanitize_package_key_component(architecture),
        sanitize_package_key_component(name),
        sanitize_package_key_component(version)
    )
}

impl Lockfile {
    pub const VERSION: u32 = 1;

    pub fn new(config_hash: String, required_packages: Vec<Arc<str>>) -> Self {
        Self {
            version: Self::VERSION,
            config_hash,
            required_packages,
            packages: HashMap::new(),
            package_groups: HashMap::new(),
        }
    }

    pub fn add_packages(
        &mut self,
        architecture: String,
        resolved_packages: &std::collections::BTreeSet<Arc<BinaryPackageControlFile<'static>>>,
        binary_packages_by_arch: &HashMap<String, Vec<crate::repository::BinaryPackage>>,
    ) -> Result<(), crate::error::AptPrepError> {
        // Create a mapping from package name+version to package key for dependency resolution
        let mut package_lookup: HashMap<(String, String), String> = HashMap::new();

        // First pass: create all package entries and build lookup map
        for control_file in resolved_packages {
            let package_name = control_file.package()?;
            let package_version = control_file.version()?;
            let _package_arch = control_file.architecture()?;

            // Generate package key
            let package_key =
                generate_package_key(&architecture, package_name, &package_version.to_string());
            package_lookup.insert(
                (package_name.to_string(), package_version.to_string()),
                package_key,
            );
        }

        // Second pass: create package entries with dependencies
        for control_file in resolved_packages {
            let package_name = control_file.package()?;
            let package_version = control_file.version()?;
            let package_arch = control_file.architecture()?;

            // Find the binary package by matching package name, version, and architecture
            let mut binary_package = None;

            // First try the package's own architecture
            if let Some(packages) = binary_packages_by_arch.get(package_arch) {
                binary_package = packages.iter().find(|pkg| {
                    let cf = &pkg.control_file;
                    cf.package().unwrap() == package_name
                        && cf.version().unwrap() == package_version
                });
            }

            // If not found and package arch is "all", try the target architecture list too
            if binary_package.is_none()
                && package_arch == "all"
                && let Some(packages) = binary_packages_by_arch.get(&architecture)
            {
                binary_package = packages.iter().find(|pkg| {
                    let cf = &pkg.control_file;
                    cf.package().unwrap() == package_name
                        && cf.version().unwrap() == package_version
                });
            }

            let binary_package =
                binary_package.ok_or_else(|| crate::error::AptPrepError::LockfileValidation {
                    details: format!(
                        "Binary package not found for {}/{}/{}",
                        package_name, package_version, package_arch
                    ),
                })?;

            let path = control_file.required_field_str("Filename")?.to_string();
            let size = control_file.field_u64("Size").ok_or_else(|| {
                crate::error::AptPrepError::LockfileValidation {
                    details: "Size field missing".to_string(),
                }
            })??;

            // Find the preferred digest
            let digest = debian_packaging::repository::release::ChecksumType::preferred_order()
                .find_map(|checksum| {
                    control_file
                        .field_str(checksum.field_name())
                        .map(|hex_digest| {
                            debian_packaging::io::ContentDigest::from_hex_digest(
                                checksum, hex_digest,
                            )
                        })
                })
                .ok_or_else(|| crate::error::AptPrepError::LockfileValidation {
                    details: "No supported digest found".to_string(),
                })?;

            // Parse dependencies and map to package keys
            let dependencies =
                self.parse_dependencies(control_file, &package_lookup, &architecture);

            // Construct the download URL
            let base_url = binary_package
                .source_info
                .url
                .as_str()
                .trim_end_matches("/");
            let download_url = if path.starts_with("/") {
                format!("{}{}", base_url, path)
            } else {
                format!("{}/{}", base_url, path.strip_prefix("./").unwrap_or(&path))
            };

            // Generate package key
            let package_key =
                generate_package_key(&architecture, package_name, &package_version.to_string());

            let lockfile_package = LockfilePackageEntry {
                name: package_name.to_string(),
                version: package_version.to_string(),
                architecture: architecture.clone(),
                download_url,
                size,
                digest: LockfileDigest::from(&digest?),
                dependencies,
            };

            // Add to packages map
            self.packages.insert(package_key.clone(), lockfile_package);

            // Add to package groups
            self.package_groups
                .entry(package_name.to_string())
                .or_default()
                .push(package_key);
        }

        Ok(())
    }

    fn parse_dependencies(
        &self,
        control_file: &BinaryPackageControlFile,
        package_lookup: &HashMap<(String, String), String>,
        _architecture: &str,
    ) -> Vec<String> {
        let mut dependencies = Vec::new();

        if let Some(depends_field) = control_file.field_str("Depends") {
            // Parse the Depends field which contains comma-separated package names with optional versions
            for dep_part in depends_field.split(',') {
                let dep_part = dep_part.trim();

                // Handle alternatives (packages separated by |)
                for alternative in dep_part.split('|') {
                    let alternative = alternative.trim();

                    // Extract just the package name (before any version constraints or parentheses)
                    if let Some(package_name) = alternative.split_whitespace().next() {
                        let package_name = package_name.trim();

                        // Remove any version constraints like (>= 1.0)
                        let package_name = if let Some(paren_pos) = package_name.find('(') {
                            &package_name[..paren_pos]
                        } else {
                            package_name
                        };

                        if !package_name.is_empty() {
                            // Try to find the package key for this dependency
                            // Note: We can't resolve exact versions here without more sophisticated dependency resolution
                            // For now, we'll just record the dependency name as a package key pattern
                            // This is a simplified approach for the initial implementation
                            for ((lookup_name, _lookup_version), package_key) in package_lookup {
                                if lookup_name == package_name
                                    && !dependencies.contains(package_key)
                                {
                                    dependencies.push(package_key.clone());
                                    break; // Only take the first match
                                }
                            }
                        }
                    }
                }
            }
        }

        dependencies
    }

    pub fn save_to_file(&self, path: &std::path::Path) -> Result<(), crate::error::AptPrepError> {
        let json = serde_json::to_string_pretty(self).map_err(|e| {
            crate::error::AptPrepError::LockfileSave {
                path: path.to_path_buf(),
                reason: format!("JSON serialization failed: {}", e),
            }
        })?;
        std::fs::write(path, json).map_err(|e| crate::error::AptPrepError::LockfileSave {
            path: path.to_path_buf(),
            reason: e.to_string(),
        })?;
        Ok(())
    }

    pub fn load_from_file(path: &std::path::Path) -> Result<Self, crate::error::AptPrepError> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            crate::error::AptPrepError::LockfileLoad {
                path: path.to_path_buf(),
                reason: e.to_string(),
            }
        })?;
        let lockfile: Lockfile = serde_json::from_str(&content).map_err(|e| {
            crate::error::AptPrepError::LockfileLoad {
                path: path.to_path_buf(),
                reason: format!("JSON parsing failed: {}", e),
            }
        })?;

        if lockfile.version != Self::VERSION {
            return Err(crate::error::AptPrepError::LockfileValidation {
                details: format!(
                    "Lockfile version {} is not supported. Expected version {}",
                    lockfile.version,
                    Self::VERSION
                ),
            });
        }

        Ok(lockfile)
    }
}
