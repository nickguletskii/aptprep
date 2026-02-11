# aptprep

A tool that resolves all Debian package dependencies needed to install a given set of Debian packages behind an air gap.

## Overview

aptprep enables air-gapped Debian package installations by:

1. **Resolving dependencies** - Creates a lockfile with all transitive dependencies resolved from Debian repositories
2. **Downloading packages** - Downloads all packages specified in the lockfile with integrity verification

The lockfile ensures reproducibility by pinning exact package versions, allowing you to recreate the same package set across different environments. This is especially useful for air-gapped environments where direct access to package repositories is not available.

## Installation

### From Release

Download the latest release from the releases page and extract:

```bash
tar -xzf aptprep-linux-x86_64.tar.gz
chmod +x aptprep
```

### Snapshot Builds (CI)

For every commit on `main` and every commit in pull requests targeting `main`, GitHub Actions publishes a downloadable snapshot artifact from the `Snapshot Build` workflow.

Download it from the workflow run's **Artifacts** section.

### From Source

```bash
cargo build --release
```

## Development Setup

### Code Formatting
This project uses automated code formatting. To set up:

1. Install dependencies: `pnpm install`
2. Configure git to use in-repo hooks: `git config core.hooksPath .githooks`

The pre-commit hook will automatically format:
- Rust code with `rustfmt`
- Bazel files with `buildifier`
- Config files with `prettier`

### Manual Formatting
Use the format script: `./scripts/format.sh`

## Usage

aptprep works with a configuration file (default: `config.yaml`) that specifies:
- Source repositories (Debian mirrors)
- Target packages to install
- Architectures to support

### 1. Create a lockfile

Resolve all dependencies and create a lockfile:

```bash
aptprep lock --config config.yaml --lockfile aptprep.lock
```

### 2. Download packages

Download all packages from the lockfile:

```bash
aptprep download --config config.yaml --lockfile aptprep.lock
```

This will download all resolved packages to the output directory specified in your configuration, ready for transfer to an air-gapped environment.

### Command Options

- `--verbose` / `-v` - Increase logging verbosity (use multiple times for more detail)
- `--config` / `-c` - Specify configuration file (default: config.yaml)
- `--lockfile` / `-l` - Specify lockfile path (default: aptprep.lock)

Run `aptprep --help` or `aptprep <command> --help` for detailed options.

## License

The source code for this project is dual-licensed under either of the following licenses, at your option:

Apache-2.0 OR MIT

The full license texts are available in the following files:
- Apache License, Version 2.0: LICENSE-APACHE
- MIT License: LICENSE-MIT

For the licenses of bundled dependencies, please refer to THIRD_PARTY_LICENSES.txt in the binary tar archives.
