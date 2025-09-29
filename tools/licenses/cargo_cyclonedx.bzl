"""Bazel rules for cargo-cyclonedx SBOM generation."""

load("@rules_rust//rust:defs.bzl", "rust_common")

def _cargo_cyclonedx_impl(ctx):
    """Implementation of the cargo_cyclonedx rule."""
    cargo_toml = ctx.file.manifest
    cargo_lock = ctx.file.lockfile
    cyclonedx_binary = ctx.file.cyclonedx_binary
    rust_toolchain = ctx.toolchains["@rules_rust//rust:toolchain_type"]

    # For workspace projects, cargo cyclonedx generates multiple SBOM files
    # We'll declare the expected output files based on the workspace structure
    expected_outputs = ctx.attr.expected_outputs if ctx.attr.expected_outputs else ["aptprep.cdx.json"]
    output_files = []

    for output_name in expected_outputs:
        output_file = ctx.actions.declare_file(output_name)
        output_files.append(output_file)

    # Set up environment for Rust toolchain and cargo-cyclonedx
    # Add the directory containing cargo-cyclonedx to PATH so cargo can find it
    cyclonedx_dir = cyclonedx_binary.dirname
    env = {
        "CARGO": rust_toolchain.cargo.path,
        "RUSTC": rust_toolchain.rustc.path,
        "PATH": "{}:/usr/bin:/bin".format(cyclonedx_dir),
    }

    # Create command to copy all generated *.cdx.json files to outputs
    # cargo cyclonedx generates files in the source directory structure, not the working directory
    copy_commands = []
    for i, output_file in enumerate(output_files):
        expected_filename = expected_outputs[i]

        # Extract crate name from filename (e.g., "aptprep-cli.cdx.json" -> "aptprep-cli")
        crate_name = expected_filename.replace(".cdx.json", "")
        source_path = "crates/{}/{}".format(crate_name, expected_filename)
        copy_commands.append("/bin/cp {} {}".format(source_path, output_file.path))

    copy_command = " && ".join(copy_commands)

    # Run cargo cyclonedx (as a cargo subcommand) and copy the generated files to declared outputs
    ctx.actions.run_shell(
        inputs = [cargo_toml, cargo_lock],
        outputs = output_files,
        tools = [rust_toolchain.cargo, rust_toolchain.rustc, cyclonedx_binary],
        env = env,
        command = "cd {} && {} cyclonedx --manifest-path {} --format json && {}".format(
            cargo_toml.dirname,
            rust_toolchain.cargo.path,
            cargo_toml.path,
            copy_command,
        ),
        mnemonic = "CargoCycloneDx",
        progress_message = "Generating SBOM for %s" % cargo_toml.short_path,
        execution_requirements = {
            "no-sandbox": "1",
        },
    )

    return DefaultInfo(files = depset(output_files))

cargo_cyclonedx = rule(
    implementation = _cargo_cyclonedx_impl,
    attrs = {
        "manifest": attr.label(
            allow_single_file = ["Cargo.toml"],
            mandatory = True,
            doc = "The Cargo.toml manifest file",
        ),
        "lockfile": attr.label(
            allow_single_file = ["Cargo.lock"],
            mandatory = True,
            doc = "The Cargo.lock file",
        ),
        "cyclonedx_binary": attr.label(
            allow_single_file = True,
            mandatory = True,
            executable = True,
            cfg = "exec",
            doc = "The cargo-cyclonedx binary to use",
        ),
        "expected_outputs": attr.string_list(
            default = [],
            doc = "List of expected output .cdx.json files. If empty, defaults to ['aptprep.cdx.json']",
        ),
    },
    toolchains = ["@rules_rust//rust:toolchain_type"],
    doc = "Generate CycloneDX SBOM from Cargo manifest using cargo-cyclonedx binary",
)
