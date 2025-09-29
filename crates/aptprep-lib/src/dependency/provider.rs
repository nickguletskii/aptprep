use super::resolver::DependencyResolutionError;
use super::types::{AptDependencyGraphElement, AptVersion, DummyPackageKey};
use crate::utils::arch_matches;
use debian_packaging::binary_package_control::BinaryPackageControlFile;
use debian_packaging::dependency::{
    DependencyVariants, DependencyVersionConstraint, SingleDependency, VersionRelationship,
};
use debian_packaging::package_version::PackageVersion;
use itertools::Itertools;
use pubgrub::{Dependencies, DependencyProvider, Map, PackageResolutionStatistics, Ranges};
use std::collections::{BTreeMap, HashMap};
use std::fmt::{Display, Formatter};
use std::sync::Arc;

// Type aliases to reduce complexity
type ProvidedByMap = HashMap<Arc<str>, Vec<(SingleDependency, Arc<str>, AptVersion)>>;

pub struct DummyPackageData {
    data_by_version: BTreeMap<AptVersion, DependenciesByVersionEntry>,
}
pub fn to_ranges(value: &DependencyVersionConstraint) -> Ranges<AptVersion> {
    match value.relationship {
        VersionRelationship::StrictlyEarlier => {
            Ranges::strictly_lower_than(AptVersion::from(&value.version))
        }
        VersionRelationship::EarlierOrEqual => Ranges::lower_than(AptVersion::from(&value.version)),
        VersionRelationship::ExactlyEqual => Ranges::singleton(AptVersion::from(&value.version)),
        VersionRelationship::LaterOrEqual => Ranges::higher_than(AptVersion::from(&value.version)),
        VersionRelationship::StrictlyLater => {
            Ranges::strictly_higher_than(AptVersion::from(&value.version))
        }
    }
}

#[derive(Clone, Debug)]
pub struct AptPackage {
    pub name: Arc<str>,
    pub dependencies_by_version: BTreeMap<AptVersion, DependenciesByVersionEntry>,
}
#[derive(Clone, Debug)]
pub struct DependenciesByVersionEntry {
    pub dependencies: Map<AptDependencyGraphElement, Ranges<AptVersion>>,
    pub control_file: Option<Arc<BinaryPackageControlFile<'static>>>,
}

impl Display for AptPackage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.name)
    }
}

impl PartialEq for AptPackage {
    fn eq(&self, other: &Self) -> bool {
        self.name.eq(&other.name)
    }
}
impl Eq for AptPackage {}

pub struct AptDependencyProvider {
    binary_packages: HashMap<Arc<str>, AptPackage>,
    pub dummy_packages: HashMap<DummyPackageKey, DummyPackageData>,
}

impl Display for AptDependencyGraphElement {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AptDependencyGraphElement::AptPackage(package) => {
                write!(f, "{}", package)
            }
            AptDependencyGraphElement::DummyPackage(dummy_key) => {
                write!(
                    f,
                    "[dummy({},{},{})]",
                    dummy_key.package_name, dummy_key.i, dummy_key.dummy_id
                )
            }
            AptDependencyGraphElement::RequestedPackages(r) => {
                write!(f, "[requested_packages({:?})]", &r.requested_packages)
            }
        }
    }
}

impl DependencyProvider for AptDependencyProvider {
    type P = AptDependencyGraphElement;
    type V = AptVersion;
    type VS = Ranges<AptVersion>;
    type Priority = (AptDependencyGraphElement, Ranges<AptVersion>);
    type M = String;
    type Err = DependencyResolutionError;

    fn prioritize(
        &self,
        package: &Self::P,
        range: &Self::VS,
        _package_conflicts_counts: &PackageResolutionStatistics,
    ) -> Self::Priority {
        // Simple strategy to make resolutions consistent
        (package.clone(), range.clone())
    }

    fn choose_version(
        &self,
        package: &Self::P,
        range: &Self::VS,
    ) -> Result<Option<Self::V>, Self::Err> {
        match package {
            AptDependencyGraphElement::AptPackage(package) => {
                let Some(package_data) = self.binary_packages.get(package.as_ref()) else {
                    tracing::error!("Package {} does not exist", package);
                    return Ok(None);
                };
                for (version, _) in package_data.dependencies_by_version.iter().rev() {
                    if range.contains(version) {
                        tracing::trace!("Choosing version {} for {}", version, package);
                        return Ok(Some(version.clone()));
                    }
                }

                tracing::error!(
                    "Package {} with constraints {} not satisfied, checked versions {}",
                    package,
                    range.to_string(),
                    package_data
                        .dependencies_by_version
                        .iter()
                        .rev()
                        .map(|(version, _)| format!("{:?}", version))
                        .join(", ")
                );
                Ok(None)
            }
            AptDependencyGraphElement::DummyPackage(dummy_package_key) => {
                for (version, _) in self.dummy_packages[dummy_package_key]
                    .data_by_version
                    .iter()
                {
                    if range.contains(version) {
                        return Ok(Some(version.clone()));
                    }
                }
                Ok(None)
            }
            &AptDependencyGraphElement::RequestedPackages(_) => Ok(Some(AptVersion::from(
                PackageVersion::parse("1.0.0").unwrap(),
            ))),
        }
    }

    fn get_dependencies(
        &self,
        package: &Self::P,
        version: &Self::V,
    ) -> Result<Dependencies<Self::P, Self::VS, Self::M>, Self::Err> {
        match package {
            AptDependencyGraphElement::AptPackage(package) => {
                let Some(package_data) = self.binary_packages.get(package.as_ref()) else {
                    return Err(DependencyResolutionError::ConfigError(
                        "Package not found".to_string(),
                    ));
                };
                let Some(control) = package_data.dependencies_by_version.get(version) else {
                    tracing::warn!(
                        "Failed to find version {} of package {}, available: {}",
                        version,
                        package,
                        package_data
                            .dependencies_by_version
                            .keys()
                            .map(|version| format!("{}", version))
                            .join(", ")
                    );
                    return Err(DependencyResolutionError::ConfigError(
                        "Version not found".to_string(),
                    ));
                };
                Ok(Dependencies::Available(control.dependencies.clone()))
            }
            AptDependencyGraphElement::DummyPackage(dummy_package_key) => {
                Ok(Dependencies::Available(
                    self.dummy_packages[dummy_package_key].data_by_version[version]
                        .dependencies
                        .clone(),
                ))
            }
            AptDependencyGraphElement::RequestedPackages(requested_packages) => {
                Ok(Dependencies::Available(
                    requested_packages
                        .requested_packages
                        .iter()
                        .map(|package| {
                            let dep = SingleDependency::parse(package)?;
                            let apt_package =
                                AptDependencyGraphElement::AptPackage(dep.package.into());
                            let version_range = dep
                                .version_constraint
                                .map(|v| to_ranges(&v))
                                .unwrap_or_else(Ranges::full);
                            // We don't needd to check the requested architecture here because `RequestedPackages` should only contain packages relevant to the architecture
                            Ok::<_, DependencyResolutionError>((apt_package, version_range))
                        })
                        .collect::<Result<_, _>>()?,
                ))
            }
        }
    }
}
impl AptDependencyProvider {
    pub fn new(
        packages: impl Iterator<Item = Arc<BinaryPackageControlFile<'static>>>,
        arch: &str,
    ) -> Result<Self, DependencyResolutionError> {
        let mut dummy_id = 0;
        let mut binary_packages: HashMap<Arc<str>, AptPackage> = HashMap::new();
        let mut dummy_packages: HashMap<DummyPackageKey, DummyPackageData> = HashMap::new();
        let binary_packages_by_package_name: HashMap<
            Arc<str>,
            Vec<Arc<BinaryPackageControlFile<'_>>>,
        > = packages.into_iter().into_group_map_by(|x| {
            Arc::from(x.package().expect("Package name not found").to_string())
        });
        let provided_by = Self::collect_virtual_packages(&binary_packages_by_package_name, arch);
        for (package_name, control_files) in binary_packages_by_package_name.iter() {
            let mut dependencies_by_version: BTreeMap<AptVersion, DependenciesByVersionEntry> =
                BTreeMap::new();
            'control: for control in control_files {
                let fields = control
                    .package_dependency_fields()
                    .expect("Failed to read package");
                let mut current_package_dependencies: Map<
                    AptDependencyGraphElement,
                    Ranges<AptVersion>,
                > = Map::default();
                for dep_list in fields
                    .pre_depends
                    .into_iter()
                    .chain(fields.depends.into_iter())
                {
                    for (dependency_seq_id, requirement, solutions) in dep_list
                        .requirements()
                        .enumerate()
                        .map(|(dependency_seq_id, requirement)| {
                            (
                                dependency_seq_id,
                                requirement,
                                Self::collect_solutions(
                                    &binary_packages_by_package_name,
                                    &provided_by,
                                    requirement,
                                    arch,
                                ),
                            )
                        })
                        .sorted_by_key(|(_dependency_seq_id, _requirement, v)| v.len())
                    {
                        if solutions.is_empty() {
                            tracing::warn!(
                                "{}:{}: Could not find any solutions for dependency {}: {:?}",
                                control.package().unwrap(),
                                control.version().unwrap(),
                                requirement.to_string(),
                                requirement,
                            );
                            continue 'control;
                        } else if solutions.len() == 1 {
                            // Simple case: no alternatives
                            let (required_name, required_range) = &solutions[0];
                            match current_package_dependencies.entry(required_name.clone()) {
                                std::collections::hash_map::Entry::Occupied(mut entry) => {
                                    let ranges = entry.get_mut();
                                    *ranges = ranges.intersection(required_range);
                                }
                                std::collections::hash_map::Entry::Vacant(entry) => {
                                    entry.insert(required_range.clone());
                                }
                            }
                        } else {
                            // Complex case: There are multiple possible packages satisfying this.
                            dummy_id += 1;

                            let mut dummy_package_dependencies: BTreeMap<
                                AptVersion,
                                DependenciesByVersionEntry,
                            > = BTreeMap::new();
                            for (j, (solution_package_name, solution_package_version_range)) in
                                solutions.into_iter().enumerate()
                            {
                                let mut virtual_res: Map<
                                    AptDependencyGraphElement,
                                    Ranges<AptVersion>,
                                > = Map::default();
                                match virtual_res.entry(solution_package_name.clone()) {
                                    std::collections::hash_map::Entry::Occupied(_entry) => {
                                        unreachable!()
                                    }
                                    std::collections::hash_map::Entry::Vacant(entry) => {
                                        entry.insert(solution_package_version_range);
                                    }
                                }
                                dummy_package_dependencies.insert(
                                    AptVersion::from(
                                        PackageVersion::parse(&format!("{}:1.0.0", j)).unwrap(),
                                    ),
                                    DependenciesByVersionEntry {
                                        control_file: None,
                                        dependencies: virtual_res,
                                    },
                                );
                            }
                            let dummy_package_key = DummyPackageKey {
                                package_name: package_name.clone(),
                                i: dependency_seq_id,
                                dummy_id,
                            };
                            dummy_packages.insert(
                                dummy_package_key.clone(),
                                DummyPackageData {
                                    data_by_version: dummy_package_dependencies,
                                },
                            );
                            current_package_dependencies.insert(
                                AptDependencyGraphElement::DummyPackage(dummy_package_key),
                                Ranges::full(),
                            );
                        }
                    }
                }
                let version = AptVersion::from(control.version().expect("Invalid package version"));
                dependencies_by_version.insert(
                    version,
                    DependenciesByVersionEntry {
                        dependencies: current_package_dependencies,
                        control_file: Some(control.clone()),
                    },
                );
            }
            if dependencies_by_version.is_empty() {
                continue;
            }
            binary_packages.insert(
                package_name.clone(),
                AptPackage {
                    name: package_name.clone(),
                    dependencies_by_version,
                },
            );
        }

        Ok(Self {
            binary_packages,
            dummy_packages,
        })
    }

    fn collect_solutions<'b>(
        binary_packages_by_package_name: &'b HashMap<Arc<str>, Vec<Arc<BinaryPackageControlFile>>>,
        provided_by: &'b ProvidedByMap,
        dependency_variants: &DependencyVariants,
        arch: &str,
    ) -> Vec<(AptDependencyGraphElement, Ranges<AptVersion>)> {
        dependency_variants
            .iter()
            .flat_map(|dependency| {
                let mut solutions: Vec<(AptDependencyGraphElement, Ranges<AptVersion>)> =
                    Vec::new();
                if !arch_matches(dependency, arch) {
                    return solutions;
                }

                if let Some(_control_files) =
                    binary_packages_by_package_name.get(dependency.package.as_str())
                {
                    // Real binary package
                    solutions.push((
                        AptDependencyGraphElement::AptPackage(Arc::from(
                            dependency.package.clone(),
                        )),
                        dependency
                            .version_constraint
                            .as_ref()
                            .map(to_ranges)
                            .unwrap_or(Ranges::full()),
                    ));
                }
                if let Some(virtual_solutions) = provided_by.get(dependency.package.as_str()) {
                    for (provided_version, provided_by, provided_by_version) in virtual_solutions {
                        // Calculate the intersection between the required and provided version ranges
                        let range = provided_version
                            .version_constraint
                            .as_ref()
                            .map(to_ranges)
                            .unwrap_or(Ranges::full())
                            .intersection(
                                &dependency
                                    .version_constraint
                                    .as_ref()
                                    .map(to_ranges)
                                    .unwrap_or(Ranges::full()),
                            );
                        if range.is_empty() {
                            // The intersection is empty, therefore this solution does not satisfy the requirements.
                            continue;
                        }
                        solutions.push((
                            AptDependencyGraphElement::AptPackage(provided_by.clone()),
                            Ranges::singleton(provided_by_version.clone()),
                        ));
                    }
                }
                solutions
            })
            .collect::<Vec<_>>()
    }

    fn collect_virtual_packages(
        grouped_packages: &HashMap<Arc<str>, Vec<Arc<BinaryPackageControlFile>>>,
        arch: &str,
    ) -> ProvidedByMap {
        let mut provided_by = HashMap::new();
        for (package_name, control_files) in grouped_packages.iter() {
            for control in control_files {
                let fields = control
                    .package_dependency_fields()
                    .expect("Failed to read package");

                let version = AptVersion::from(control.version().expect("Invalid package version"));
                if let Some(provides) = &fields.provides {
                    for virtual_package in provides.requirements().flat_map(|v| v.iter()) {
                        if !arch_matches(virtual_package, arch) {
                            continue;
                        }

                        provided_by
                            .entry(Arc::from(virtual_package.package.clone()))
                            .or_insert_with(Vec::new)
                            .push((
                                virtual_package.clone(),
                                package_name.clone(),
                                version.clone(),
                            ));
                    }
                }
            }
        }
        provided_by
    }
    pub fn get_control(
        &self,
        package_name: &str,
        apt_version: &AptVersion,
    ) -> Option<&Arc<BinaryPackageControlFile<'static>>> {
        let package = self.binary_packages.get(package_name)?;
        let deps = package.dependencies_by_version.get(apt_version)?;
        deps.control_file.as_ref()
    }
}
