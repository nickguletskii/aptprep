use crate::error::AptPrepError;
use debian_packaging::dependency::{
    DependencyVersionConstraint, SingleDependency, VersionRelationship,
};
use debian_packaging::package_version::PackageVersion;
use std::cmp::Ordering;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum PackageNameAndVersion {
    RangeStart {
        name: String,
    },
    Versioned {
        name: String,
        version: String,
        parsed_version: PackageVersion,
    },
    RangeEnd {
        name: String,
    },
}

impl PackageNameAndVersion {
    fn validate_component(component_name: &str, value: &str) -> Result<(), AptPrepError> {
        if value.is_empty() {
            Err(AptPrepError::LockfileValidation {
                details: format!("{component_name} cannot be empty"),
            })
        } else {
            Ok(())
        }
    }

    fn from_parts(
        name: String,
        version: String,
        parsed_version: PackageVersion,
    ) -> Result<Self, AptPrepError> {
        Self::validate_component("package name", &name)?;
        Self::validate_component("package version", &version)?;
        Ok(Self::Versioned {
            name,
            version,
            parsed_version,
        })
    }

    pub fn from_control_file(name: &str, version: &PackageVersion) -> Result<Self, AptPrepError> {
        let version_string = version.to_string();
        Self::from_parts(name.to_string(), version_string, version.clone())
    }

    pub fn range_start(name: &str) -> Self {
        Self::RangeStart {
            name: name.to_string(),
        }
    }

    pub fn range_end(name: &str) -> Self {
        Self::RangeEnd {
            name: name.to_string(),
        }
    }

    pub fn satisfies_dependency(&self, dependency: &SingleDependency) -> bool {
        match self {
            Self::Versioned {
                name,
                parsed_version,
                ..
            } => {
                if name.as_str() != dependency.package {
                    return false;
                }

                match &dependency.version_constraint {
                    Some(constraint) => Self::satisfies_constraint(parsed_version, constraint),
                    None => true,
                }
            }
            Self::RangeStart { .. } | Self::RangeEnd { .. } => false,
        }
    }

    fn name(&self) -> &str {
        match self {
            Self::RangeStart { name } => name,
            Self::Versioned { name, .. } => name,
            Self::RangeEnd { name } => name,
        }
    }

    fn kind_rank(&self) -> u8 {
        match self {
            Self::RangeStart { .. } => 0,
            Self::Versioned { .. } => 1,
            Self::RangeEnd { .. } => 2,
        }
    }

    fn version_sort_key(&self) -> Option<(&PackageVersion, &str)> {
        match self {
            Self::Versioned {
                parsed_version,
                version,
                ..
            } => Some((parsed_version, version)),
            _ => None,
        }
    }

    fn satisfies_constraint(
        parsed_version: &PackageVersion,
        constraint: &DependencyVersionConstraint,
    ) -> bool {
        match constraint.relationship {
            VersionRelationship::StrictlyEarlier => parsed_version < &constraint.version,
            VersionRelationship::EarlierOrEqual => parsed_version <= &constraint.version,
            VersionRelationship::ExactlyEqual => parsed_version == &constraint.version,
            VersionRelationship::LaterOrEqual => parsed_version >= &constraint.version,
            VersionRelationship::StrictlyLater => parsed_version > &constraint.version,
        }
    }
}

impl Ord for PackageNameAndVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name()
            .cmp(other.name())
            .then_with(|| self.kind_rank().cmp(&other.kind_rank()))
            .then_with(
                || match (self.version_sort_key(), other.version_sort_key()) {
                    (Some((l_parsed, l_str)), Some((r_parsed, r_str))) => {
                        r_parsed.cmp(l_parsed).then_with(|| l_str.cmp(r_str))
                    }
                    _ => Ordering::Equal,
                },
            )
    }
}

impl PartialOrd for PackageNameAndVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
