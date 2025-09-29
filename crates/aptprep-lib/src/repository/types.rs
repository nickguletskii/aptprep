use debian_packaging::binary_package_control::BinaryPackageControlFile;
use reqwest::Url;
use std::sync::Arc;

#[derive(Debug)]
pub struct SourceInfo {
    pub url: Url,
}

#[derive(Debug, Clone)]
pub struct BinaryPackage {
    pub control_file: Arc<BinaryPackageControlFile<'static>>,
    pub source_info: Arc<SourceInfo>,
}

impl BinaryPackage {
    pub fn key(&self) -> &BinaryPackageControlFile<'_> {
        &self.control_file
    }
}

pub fn iterate_all_relevant_packages<'a>(
    binary_packages: &'a std::collections::HashMap<String, Vec<BinaryPackage>>,
    architecture: &'a String,
) -> impl Iterator<Item = &'a BinaryPackage> + 'a {
    binary_packages
        .get(architecture)
        .map(|v| v.as_slice())
        .unwrap_or_default()
        .iter()
        .chain(
            binary_packages
                .get("all")
                .map(|v| v.as_slice())
                .unwrap_or_default(),
        )
}
