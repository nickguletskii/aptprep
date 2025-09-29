use crate::config::OutputConfig;
use crate::download::DownloadItem;
use crate::error::AptPrepError;
use crate::lockfile::Lockfile;
use crate::repository::BinaryPackage;
use debian_packaging::binary_package_control::BinaryPackageControlFile;
use debian_packaging::control::{ControlFile, ControlParagraph};
use debian_packaging::error::DebianError;
use debian_packaging::io::ContentDigest;
use debian_packaging::repository::builder::DebPackageReference;
use debian_packaging::repository::release::ChecksumType;
use std::collections::{BTreeSet, HashMap};
use std::io::BufWriter;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::Arc;

pub fn generate_packages_file(
    collected_packages: &BTreeSet<Arc<BinaryPackageControlFile<'static>>>,
    binary_packages_by_control_file: &HashMap<
        Arc<BinaryPackageControlFile<'static>>,
        &BinaryPackage,
    >,
    output_config: &OutputConfig,
) -> Result<(Vec<DownloadItem>, PathBuf), AptPrepError> {
    let mut fetches = Vec::new();
    let mut control_file = ControlFile::default();

    for cf in collected_packages {
        let Ok(_filename) = cf.deb_filename() else {
            tracing::warn!("Skipping package, no Debian package name specified");
            continue;
        };
        let path = cf.required_field_str("Filename")?.to_string();

        let _size = cf
            .field_u64("Size")
            .ok_or_else(|| DebianError::ControlRequiredFieldMissing("Size".to_string()))??;

        let digest = ChecksumType::preferred_order()
            .find_map(|checksum| {
                cf.field_str(checksum.field_name())
                    .map(|hex_digest| ContentDigest::from_hex_digest(checksum, hex_digest))
            })
            .ok_or(DebianError::RepositoryReadCouldNotDeterminePackageDigest)??;

        let package = binary_packages_by_control_file
            .get(cf)
            .expect("Failed to get binary package");

        let url = package.source_info.url.join(&path).expect("Invalid URL");
        let filename = url
            .path_segments()
            .and_then(|mut segments| segments.next_back())
            .ok_or_else(|| AptPrepError::Download {
                message: "Invalid URL: no filename in path".to_string(),
            })?;

        fetches.push(DownloadItem {
            base_url: package
                .source_info
                .url
                .to_string()
                .trim_end_matches("/")
                .to_string(),
            rel_path: format!("/{}", path.strip_prefix("./").unwrap_or(&path)),
            size: cf.deb_size_bytes().ok(),
            digest,
            output_path: Some(filename.to_string()),
        });

        let mut paragraph: ControlParagraph<'_> = cf.as_ref().deref().clone();
        paragraph.set_field_from_string("Filename".into(), format!("./{}", filename).into());
        control_file.add_paragraph(paragraph);
    }

    let packages_path = output_config.path.join("Packages");

    std::fs::create_dir_all(output_config.path.as_path()).map_err(|e| {
        AptPrepError::DownloadDirectoryCreation {
            path: output_config.path.clone(),
            reason: e.to_string(),
        }
    })?;
    let packages_file = std::fs::File::create(&packages_path).map_err(AptPrepError::Io)?;
    let mut writer = BufWriter::new(packages_file);
    control_file.write(&mut writer).map_err(AptPrepError::Io)?;

    Ok((fetches, packages_path))
}

pub fn generate_packages_file_from_lockfile(
    lockfile: &Lockfile,
    output_config: &OutputConfig,
) -> Result<PathBuf, AptPrepError> {
    let mut control_file = ControlFile::default();

    for lockfile_package in lockfile.packages.values() {
        // Create a control paragraph from the package information
        let mut paragraph = ControlParagraph::default();

        // Extract filename from download URL
        let filename = lockfile_package
            .download_url
            .split('/')
            .next_back()
            .ok_or_else(|| AptPrepError::Download {
                message: format!("Invalid download URL: {}", lockfile_package.download_url),
            })?;

        // Set basic package fields
        paragraph.set_field_from_string("Package".into(), lockfile_package.name.clone().into());
        paragraph.set_field_from_string("Version".into(), lockfile_package.version.clone().into());
        paragraph.set_field_from_string(
            "Architecture".into(),
            lockfile_package.architecture.clone().into(),
        );
        paragraph.set_field_from_string("Size".into(), lockfile_package.size.to_string().into());
        paragraph.set_field_from_string("Filename".into(), format!("./{}", filename).into());

        // Set digest field based on algorithm
        match lockfile_package.digest.algorithm.as_str() {
            "SHA256" => {
                paragraph.set_field_from_string(
                    "SHA256".into(),
                    lockfile_package.digest.value.clone().into(),
                );
            }
            "SHA1" => {
                paragraph.set_field_from_string(
                    "SHA1".into(),
                    lockfile_package.digest.value.clone().into(),
                );
            }
            "MD5Sum" => {
                paragraph.set_field_from_string(
                    "MD5sum".into(),
                    lockfile_package.digest.value.clone().into(),
                );
            }
            _ => {
                return Err(AptPrepError::PackageValidation {
                    package: lockfile_package.name.clone(),
                    details: format!(
                        "Unsupported digest algorithm: {}",
                        lockfile_package.digest.algorithm
                    ),
                });
            }
        }

        control_file.add_paragraph(paragraph);
    }

    let packages_path = output_config.path.join("Packages");

    std::fs::create_dir_all(output_config.path.as_path()).map_err(|e| {
        AptPrepError::DownloadDirectoryCreation {
            path: output_config.path.clone(),
            reason: e.to_string(),
        }
    })?;
    let packages_file = std::fs::File::create(&packages_path).map_err(AptPrepError::Io)?;
    let mut writer = BufWriter::new(packages_file);
    control_file.write(&mut writer).map_err(AptPrepError::Io)?;

    Ok(packages_path)
}
