"""
Reusable Starlark macro for Bazel end-to-end APT repository tests using Podman.

This macro defines a test target that:
1. Runs the aptprep `lock` command with a configuration file
2. Runs the aptprep `download` command to populate a local APT repository
3. Spins up a Podman Ubuntu container with the local repository
4. Disables online APT repositories
5. Verifies package installation from the local repository

Usage:
    load("//e2e:podman_apt_test.bzl", "podman_apt_e2e_test")

    podman_apt_e2e_test(
        name = "test_packages",
        config = "//e2e:test_config.yaml",
        packages = ["curl", "vim", "git"],
        container_name = "test-apt-repo",
    )
"""

def _expand_test_script_impl(ctx):
    """Implementation of the script expansion rule."""
    output = ctx.outputs.out

    # Resolve the actual file references
    config_file = ctx.file.config
    binary_file = ctx.file.binary

    # Format packages list as space-separated string for shell
    packages = ctx.attr.packages
    packages_str = " ".join(packages) if packages else "curl"

    # Compute relative path from this script to the binary
    # Script is in e2e package, binary is in crates/aptprep-cli package
    # In runfiles, both will have _main/ prefix
    # Script location: _main/e2e/podman_apt_repo_test_script_gen.sh
    # Binary location: _main/crates/aptprep-cli/aptprep
    # From e2e/ to crates/aptprep-cli/, we go up 1 level then into crates
    # Relative path: ../crates/aptprep-cli/aptprep
    config_basename = config_file.basename
    binary_basename = binary_file.basename

    # Create substitutions for the template
    substitutions = {
        "{CONFIG}": config_basename,
        "{BINARY}": "../crates/aptprep-cli/" + binary_basename,
        "{CONTAINER_NAME}": ctx.attr.container_name,
        "{PACKAGES}": packages_str,
    }

    # Expand the template
    ctx.actions.expand_template(
        template = ctx.file.template,
        output = output,
        substitutions = substitutions,
    )

    return [DefaultInfo(files = depset([output]))]

_expand_test_script = rule(
    implementation = _expand_test_script_impl,
    attrs = {
        "template": attr.label(
            allow_single_file = True,
            mandatory = True,
            doc = "The template file to expand",
        ),
        "config": attr.label(
            allow_single_file = True,
            mandatory = True,
            doc = "The config YAML file",
        ),
        "binary": attr.label(
            allow_single_file = True,
            mandatory = True,
            doc = "The aptprep binary",
        ),
        "container_name": attr.string(
            default = "ubuntu-apt-repo",
            doc = "Name of the Podman container",
        ),
        "packages": attr.string_list(
            default = ["curl"],
            doc = "List of packages to test installation",
        ),
    },
    outputs = {
        "out": "%{name}.sh",
    },
)

def podman_apt_e2e_test(
        name,
        config,
        packages,
        container_name = "ubuntu-apt-repo",
        timeout = "long",
        binary = "//crates/aptprep-cli:aptprep"):
    """
    Create an end-to-end Podman-based APT repository test.

    This macro generates a test that:
    1. Uses the aptprep binary to generate a lockfile from config
    2. Downloads packages to a local repository
    3. Sets up a Podman container with the local repo
    4. Verifies package installation

    Args:
        name: Name of the test target
        config: Label pointing to the config YAML file (e.g., "//e2e:test_config.yaml")
        packages: List of package names to verify installation
        container_name: Name of the Podman container (default: "ubuntu-apt-repo")
        timeout: Timeout for the test (default: "long")
        binary: Label pointing to the aptprep binary (default: "//crates/aptprep-cli:aptprep")

    Returns:
        Creates sh_test and rule targets
    """

    # Create the test script from the template
    _expand_test_script(
        name = name + "_script_gen",
        template = "//e2e:test_script.sh.tpl",
        config = config,
        binary = binary,
        container_name = container_name,
        packages = packages,
    )

    # Create the test target
    native.sh_test(
        name = name,
        srcs = [name + "_script_gen"],
        timeout = timeout,
        data = [config, binary],
        tags = [
            "e2e",
            "requires-podman",
            "manual",  # Requires Podman, not suitable for all CI environments
        ],
    )
