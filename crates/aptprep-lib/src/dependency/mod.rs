mod provider;
mod resolver;
mod types;

pub use provider::AptDependencyProvider;
pub use resolver::{DependencyResolutionError, resolve_dependencies};
pub use types::{AptDependencyGraphElement, AptVersion};
