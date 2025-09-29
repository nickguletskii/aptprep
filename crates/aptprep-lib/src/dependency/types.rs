use debian_packaging::package_version::PackageVersion;
use std::fmt::{Display, Formatter};
use std::sync::Arc;

#[derive(Clone, Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct AptVersion(pub Arc<PackageVersion>);

impl<'a> From<&'a PackageVersion> for AptVersion {
    fn from(value: &'a PackageVersion) -> Self {
        Self(Arc::from(value.clone()))
    }
}

impl From<PackageVersion> for AptVersion {
    fn from(value: PackageVersion) -> Self {
        Self(Arc::from(value))
    }
}

impl Display for AptVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}-{}",
            self.0.epoch_assumed(),
            self.0.upstream_version(),
            self.0.debian_revision().unwrap_or("")
        )
    }
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct DummyPackageKey {
    pub package_name: Arc<str>,
    pub i: usize,
    pub dummy_id: usize,
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum AptDependencyGraphElement {
    AptPackage(Arc<str>),
    DummyPackage(DummyPackageKey),
    RequestedPackages(Arc<RequestedPackages>),
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct RequestedPackages {
    pub requested_packages: Vec<Arc<str>>,
}

impl<T: IntoIterator<Item = Arc<str>>> From<T> for RequestedPackages {
    fn from(value: T) -> Self {
        Self {
            requested_packages: value.into_iter().collect(),
        }
    }
}
