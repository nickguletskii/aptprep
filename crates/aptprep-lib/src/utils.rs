use debian_packaging::dependency::SingleDependency;

pub fn arch_matches(dep: &SingleDependency, architecture: &str) -> bool {
    if let Some((negate, arches)) = &dep.architectures {
        let contains = arches
            .iter()
            .any(|x| x == architecture || x == "all" || x == "any");

        // Requesting an arch mismatch.
        if (*negate && contains) || (!*negate && !contains) {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arch_matches_no_architecture_specified() {
        let dep = SingleDependency {
            package: "test-package".to_string(),
            version_constraint: None,
            architectures: None,
        };

        assert!(arch_matches(&dep, "amd64"));
        assert!(arch_matches(&dep, "arm64"));
        assert!(arch_matches(&dep, "any"));
    }

    #[test]
    fn test_arch_matches_with_specific_architecture() {
        let dep = SingleDependency {
            package: "test-package".to_string(),
            version_constraint: None,
            architectures: Some((false, vec!["amd64".to_string()])),
        };

        assert!(arch_matches(&dep, "amd64"));
        assert!(!arch_matches(&dep, "arm64"));
    }

    #[test]
    fn test_arch_matches_with_all_architecture() {
        let dep = SingleDependency {
            package: "test-package".to_string(),
            version_constraint: None,
            architectures: Some((false, vec!["all".to_string()])),
        };

        assert!(arch_matches(&dep, "amd64"));
        assert!(arch_matches(&dep, "arm64"));
        assert!(arch_matches(&dep, "any"));
    }

    #[test]
    fn test_arch_matches_with_negated_architecture() {
        let dep = SingleDependency {
            package: "test-package".to_string(),
            version_constraint: None,
            architectures: Some((true, vec!["amd64".to_string()])),
        };

        assert!(!arch_matches(&dep, "amd64"));
        assert!(arch_matches(&dep, "arm64"));
    }

    #[test]
    fn test_arch_matches_with_multiple_architectures() {
        let dep = SingleDependency {
            package: "test-package".to_string(),
            version_constraint: None,
            architectures: Some((false, vec!["amd64".to_string(), "arm64".to_string()])),
        };

        assert!(arch_matches(&dep, "amd64"));
        assert!(arch_matches(&dep, "arm64"));
        assert!(!arch_matches(&dep, "i386"));
    }
}
