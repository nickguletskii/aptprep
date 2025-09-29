use base64::Engine;
use clap::Parser;
use eyre::{bail, Context, ContextCompat, Result};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use spdx::Expression;
use std::{
    collections::HashMap,
    fs,
    io::Write,
    path::Path,
};

#[derive(Parser, Debug)]
#[command(name = "extract_licenses")]
#[command(about = "Extract and bundle licenses from CycloneDX SBOM")]
struct Args {
    /// Paths to the CycloneDX SBOM files
    cdx_files: Vec<String>,
    /// Path to the SPDX license repository
    #[arg(short, long)]
    spdx_repo: String,
    /// Output file path for bundled licenses
    #[arg(short, long)]
    output_file: String,
}

#[derive(Debug, Deserialize)]
struct CdxDocument {
    components: Option<Vec<Component>>,
}

#[derive(Debug, Deserialize)]
struct Component {
    purl: Option<String>,
    name: Option<String>,
    version: Option<String>,
    author: Option<String>,
    licenses: Option<Vec<LicenseChoice>>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum LicenseChoice {
    Expression { expression: String },
    License { license: CdxLicenseData },
}

#[derive(Debug, Deserialize)]
struct CdxLicenseData {
    id: Option<String>,
    name: Option<String>,
    text: Option<TextData>,
}

#[derive(Debug, Deserialize)]
struct TextData {
    content: String,
    encoding: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum LicenseData {
    Spdx {
        license_id: String,
        resolved_text: Option<String>,
    },
    Custom {
        name: String,
        text: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ComponentInfo {
    purl: String,
    name: String,
    version: String,
    author: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ComponentLicenseInfo {
    component: ComponentInfo,
    original_expression: String,
}

impl LicenseData {
    fn display_name(&self) -> &str {
        match self {
            Self::Spdx { license_id, .. } => license_id,
            Self::Custom { name, .. } => name,
        }
    }

    fn get_text(&self) -> Option<&str> {
        match self {
            Self::Spdx { resolved_text, .. } => resolved_text.as_deref(),
            Self::Custom { text, .. } => Some(text),
        }
    }
}


fn parse_license_expression(
    license_str: String,
    license_text: Option<String>,
    spdx_repo_path: &Path,
    license_data_map: &mut HashMap<String, LicenseData>,
) -> Result<Vec<String>> {
    Ok(Expression::parse(&license_str)
        .map(|expr| parse_spdx_expression(expr, spdx_repo_path, license_data_map))
        .unwrap_or_else(|_| parse_custom_license(license_str, license_text, license_data_map)))
}

fn parse_spdx_expression(
    expr: Expression,
    spdx_repo_path: &Path,
    license_data_map: &mut HashMap<String, LicenseData>,
) -> Vec<String> {
    let mut individual_licenses = Vec::new();
    let _ = expr.evaluate_with_failures(|license| {
        individual_licenses.push(license.license.to_string());
        true
    });

    for license_id in &individual_licenses {
        license_data_map.entry(license_id.clone()).or_insert_with(|| {
            let license_file = spdx_repo_path
                .join("text")
                .join(format!("{license_id}.txt"));

            LicenseData::Spdx {
                license_id: license_id.clone(),
                resolved_text: fs::read_to_string(&license_file).ok(),
            }
        });
    }

    individual_licenses
}

fn parse_custom_license(
    license_str: String,
    license_text: Option<String>,
    license_data_map: &mut HashMap<String, LicenseData>,
) -> Vec<String> {
    let text = license_text.unwrap_or_else(|| {
        format!("Custom license: {license_str}\n(No license text available)")
    });

    let unique_id = generate_custom_license_id(&license_str, &text);

    license_data_map.entry(unique_id.clone()).or_insert_with(|| {
        LicenseData::Custom {
            name: license_str,
            text,
        }
    });

    vec![unique_id]
}

fn generate_custom_license_id(license_str: &str, text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    let hash = format!("{:x}", hasher.finalize());
    format!("custom_{license_str}_{}", &hash[..8])
}

fn process_license_choice(
    license_choice: &LicenseChoice,
    component_info: &ComponentInfo,
    component_name: &str,
    component_idx: usize,
    license_idx: usize,
    spdx_repo_path: &Path,
    license_data_map: &mut HashMap<String, LicenseData>,
    license_to_component_expressions: &mut HashMap<String, Vec<ComponentLicenseInfo>>,
) -> Result<Vec<String>> {
    let (license_str, license_text) = extract_license_info(license_choice, component_name, component_idx, license_idx)?;

    let license_ids = parse_license_expression(
        license_str.clone(),
        license_text,
        spdx_repo_path,
        license_data_map,
    )
    .with_context(|| {
        format!(
            "Failed to parse license expression '{}' in component '{}' (index {}), license {}",
            license_str, component_name, component_idx, license_idx
        )
    })?;

    let component_license_info = ComponentLicenseInfo {
        component: component_info.clone(),
        original_expression: license_str,
    };

    for license_id in &license_ids {
        license_to_component_expressions
            .entry(license_id.clone())
            .or_insert_with(Vec::new)
            .push(component_license_info.clone());
    }

    Ok(license_ids)
}

fn extract_license_info(
    license_choice: &LicenseChoice,
    component_name: &str,
    component_idx: usize,
    license_idx: usize,
) -> Result<(String, Option<String>)> {
    match license_choice {
        LicenseChoice::Expression { expression } => Ok((expression.clone(), None)),
        LicenseChoice::License { license } => {
            let license_str = license
                .id
                .clone()
                .or_else(|| license.name.clone())
                .with_context(|| {
                    format!(
                        "No license ID or name found in component '{}' (index {}), license {}",
                        component_name, component_idx, license_idx
                    )
                })?;

            let license_text = license.text.as_ref().map(|text_data| {
                if text_data.encoding.as_deref() == Some("base64") {
                    base64::engine::general_purpose::STANDARD
                        .decode(&text_data.content)
                        .with_context(|| {
                            format!(
                                "Failed to decode base64 license text for '{}' in component '{}' (index {}), license {}",
                                license_str, component_name, component_idx, license_idx
                            )
                        })
                        .and_then(|decoded| {
                            String::from_utf8(decoded).with_context(|| {
                                format!(
                                    "Failed to convert decoded license text to UTF-8 for '{}' in component '{}' (index {}), license {}",
                                    license_str, component_name, component_idx, license_idx
                                )
                            })
                        })
                } else {
                    Ok(text_data.content.clone())
                }
            }).transpose()?;

            Ok((license_str, license_text))
        }
    }
}

fn extract_license_data(
    cdx_path: &Path,
    spdx_repo_path: &Path,
) -> Result<(
    HashMap<ComponentInfo, Vec<String>>,
    HashMap<String, LicenseData>,
    HashMap<String, Vec<ComponentLicenseInfo>>,
)> {
    let content = fs::read_to_string(cdx_path)
        .with_context(|| format!("Failed to read CDX file: {}", cdx_path.display()))?;

    let cdx_data: CdxDocument = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse CDX file: {}", cdx_path.display()))?;

    let mut component_to_licenses = HashMap::new();
    let mut license_data_map = HashMap::new();
    let mut license_to_component_expressions = HashMap::new();

    let components = cdx_data.components.unwrap_or_default();

    for (component_idx, component) in components.iter().enumerate() {
        let purl = component.purl.as_deref().unwrap_or("");
        if purl.is_empty() {
            continue;
        }

        let component_info = ComponentInfo {
            purl: purl.to_string(),
            name: component.name.as_deref().unwrap_or("").to_string(),
            version: component.version.as_deref().unwrap_or("").to_string(),
            author: component.author.as_deref().unwrap_or("").to_string(),
        };

        let component_name_fallback = format!("component_{}", component_idx);
        let component_name = component.name.as_deref().unwrap_or(&component_name_fallback);

        let component_license_ids: Result<Vec<String>> = component.licenses
            .as_ref()
            .map(|licenses| {
                licenses.iter()
                    .enumerate()
                    .map(|(license_idx, license_choice)| {
                        process_license_choice(
                            license_choice,
                            &component_info,
                            component_name,
                            component_idx,
                            license_idx,
                            spdx_repo_path,
                            &mut license_data_map,
                            &mut license_to_component_expressions,
                        )
                    })
                    .collect::<Result<Vec<_>>>()
                    .map(|nested| nested.into_iter().flatten().collect())
            })
            .unwrap_or_else(|| Ok(Vec::new()));

        let license_ids = component_license_ids?;
        if !license_ids.is_empty() {
            component_to_licenses.insert(component_info, license_ids);
        }
    }

    Ok((
        component_to_licenses,
        license_data_map,
        license_to_component_expressions,
    ))
}

fn write_component_info<W: Write>(
    writer: &mut W,
    component_license_info: &ComponentLicenseInfo,
    license_id: &str,
) -> Result<()> {
    let component = &component_license_info.component;
    write!(writer, "  - {} {}", component.name, component.version)?;

    if !component.author.is_empty() {
        write!(writer, " (by {})", component.author)?;
    }

    if component_license_info.original_expression != license_id {
        write!(writer, " [expression: {}]", component_license_info.original_expression)?;
    }

    writeln!(writer, " [{}]", component.purl)?;
    Ok(())
}

fn write_license_section<W: Write>(
    writer: &mut W,
    license_id: &str,
    license_data: &LicenseData,
    component_license_infos: Option<&Vec<ComponentLicenseInfo>>,
) -> Result<()> {
    writeln!(writer, "----------------------------------------")?;
    writeln!(writer, "License: {}", license_data.display_name())?;

    if let Some(infos) = component_license_infos {
        writeln!(writer, "Applicable to packages:")?;
        let mut sorted_infos: Vec<_> = infos.iter().collect();
        sorted_infos.sort_by_key(|cli| &cli.component.name);

        for info in sorted_infos {
            write_component_info(writer, info, license_id)?;
        }
    }

    writeln!(writer, "----------------------------------------\n")?;

    if let Some(text) = license_data.get_text() {
        write!(writer, "{}", text)?;
        if !text.ends_with('\n') {
            writeln!(writer)?;
        }
        writeln!(writer)?;
    }

    Ok(())
}

fn bundle_licenses(cdx_paths: &[&Path], spdx_repo_path: &Path, output_path: &Path) -> Result<()> {
    let mut combined_component_to_licenses = HashMap::new();
    let mut combined_license_data_map = HashMap::new();
    let mut combined_license_to_component_expressions = HashMap::new();

    // Process each SBOM file and merge the results
    for cdx_path in cdx_paths {
        let (component_to_licenses, license_data_map, license_to_component_expressions) =
            extract_license_data(cdx_path, spdx_repo_path)
                .with_context(|| format!("Failed to extract license data from CDX: {}", cdx_path.display()))?;

        // Merge component_to_licenses
        for (component, licenses) in component_to_licenses {
            combined_component_to_licenses.insert(component, licenses);
        }

        // Merge license_data_map
        for (license_id, license_data) in license_data_map {
            combined_license_data_map.insert(license_id, license_data);
        }

        // Merge license_to_component_expressions
        for (license_id, component_infos) in license_to_component_expressions {
            combined_license_to_component_expressions
                .entry(license_id)
                .or_insert_with(Vec::new)
                .extend(component_infos);
        }
    }

    let (component_to_licenses, license_data_map, license_to_component_expressions) =
        (combined_component_to_licenses, combined_license_data_map, combined_license_to_component_expressions);

    // Create license-to-components mapping for output generation
    let mut license_to_components: HashMap<&String, Vec<&ComponentInfo>> = HashMap::new();
    for (component, license_ids) in &component_to_licenses {
        for license_id in license_ids {
            license_to_components
                .entry(license_id)
                .or_insert_with(Vec::new)
                .push(component);
        }
    }

    // Categorize licenses
    let mut spdx_licenses: Vec<&String> = Vec::new();
    let mut custom_licenses: Vec<&String> = Vec::new();
    let mut missing_spdx_licenses: Vec<&String> = Vec::new();

    for (license_id, license_data) in &license_data_map {
        match license_data {
            LicenseData::Spdx { resolved_text, .. } => {
                if resolved_text.is_some() {
                    spdx_licenses.push(license_id);
                } else {
                    missing_spdx_licenses.push(license_id);
                }
            }
            LicenseData::Custom { .. } => {
                custom_licenses.push(license_id);
            }
        }
    }

    // Generate output
    let mut output = Vec::new();

    let sbom_names: Vec<String> = cdx_paths
        .iter()
        .map(|path| path.file_name().unwrap_or_default().to_string_lossy().to_string())
        .collect();

    writeln!(
        output,
        "Third-Party Software Licenses\n\
         =============================\n\n\
         This file contains the licenses for all third-party software used in this project.\n\
         Generated from SBOMs: {}\n\
         Using SPDX repository: {}\n",
        sbom_names.join(", "),
        spdx_repo_path.display()
    )?;

    let mut found_licenses = 0;

    // Process licenses with available text
    let all_licenses_with_text = [&spdx_licenses[..], &custom_licenses[..]].concat();
    let mut sorted_licenses = all_licenses_with_text;
    sorted_licenses.sort();

    for license_id in &sorted_licenses {
        if let Some(license_data) = license_data_map.get(*license_id) {
            if license_data.get_text().is_some() {
                let component_infos = license_to_component_expressions.get(*license_id);
                write_license_section(&mut output, license_id, license_data, component_infos)?;
                found_licenses += 1;
            }
        }
    }

    // Report missing SPDX licenses
    if !missing_spdx_licenses.is_empty() {
        writeln!(
            output,
            "----------------------------------------\n\
             Missing SPDX License Files\n\
             ----------------------------------------\n"
        )?;

        missing_spdx_licenses.sort();
        for license_id in &missing_spdx_licenses {
            writeln!(output, "- {}", license_id)?;

            if let Some(component_license_infos) = license_to_component_expressions.get(*license_id) {
                writeln!(output, "  Used by packages:")?;
                let mut sorted_component_infos: Vec<_> = component_license_infos.iter().collect();
                sorted_component_infos.sort_by_key(|cli| &cli.component.name);

                for component_license_info in sorted_component_infos {
                    write!(output, "  ")?;  // Add extra indentation
                    write_component_info(&mut output, component_license_info, license_id)?;
                }
            }
        }
        writeln!(output)?;
    }

    // Write output file
    fs::write(output_path, output)
        .with_context(|| format!("Failed to write output file: {}", output_path.display()))?;

    println!(
        "Bundled {} license files into {}",
        found_licenses,
        output_path.display()
    );

    if !missing_spdx_licenses.is_empty() {
        let missing_list: Vec<String> = missing_spdx_licenses.iter().map(|s| s.to_string()).collect();
        bail!(
            "Error: {} SPDX license files not found: {}",
            missing_spdx_licenses.len(),
            missing_list.join(", ")
        );
    }

    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();

    if args.cdx_files.is_empty() {
        bail!("At least one CycloneDX SBOM file must be provided");
    }

    let cdx_paths: Vec<&Path> = args.cdx_files.iter().map(|s| Path::new(s)).collect();
    let spdx_repo_path = Path::new(&args.spdx_repo);
    let output_path = Path::new(&args.output_file);

    bundle_licenses(&cdx_paths, spdx_repo_path, output_path)?;

    Ok(())
}
