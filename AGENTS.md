# aptprep - Claude Code Project Context

A Rust tool for resolving and downloading Debian package dependencies for air-gapped installations.

## Quick Start

```bash
# Build
cargo build

# Run tests
cargo test

# Run the CLI
cargo run -- --help
cargo run -- lock --config config.yaml --lockfile aptprep.lock  # Resolve dependencies
cargo run -- download --config config.yaml --lockfile aptprep.lock  # Download packages
cargo run -- generate_packages_file_from_lockfile --config config.yaml --lockfile aptprep.lock  # Generate Packages index

# Format code (or use git hooks for automatic formatting)
./scripts/format.sh

# Lint
cargo clippy -- -D warnings
```

## Architecture

**Workspace Structure** (Cargo workspace with 3 crates):

- `crates/aptprep-lib/` - Core library (dependency resolution, package downloading)
- `crates/aptprep-cli/` - CLI interface and argument parsing
- `crates/aptprep-e2e-tests/` - End-to-end integration tests

**Key Components**:

- Dependency resolution using `pubgrub` solver
- HTTP downloading with `reqwest` and `opendal`
- Debian package parsing via `debian-packaging` crate
- Lockfile serialization with integrity verification

**Dual Build System**:

- **Cargo** (primary): Use for all development work (`cargo build`, `cargo test`)
- **Bazel** (releases only): Used exclusively for GitHub releases with cross-compilation, SBOM generation, and license bundling
  - Test release builds: `bazel build //:aptprep_linux_x86_64`
  - Do NOT use Bazel for day-to-day development

## Development Setup

### Initial Setup

```bash
# Install Node.js dependencies (for formatting tools)
pnpm install

# Configure git hooks for automatic formatting on commit
git config core.hooksPath .githooks
```

### Verify Setup

```bash
cargo build
cargo test
```

## Code Style

### Formatting (Automated)

The git hooks automatically format on commit:

- **Rust**: `rustfmt` with `rustfmt.toml`
- **Bazel**: `buildifier`
- **Config files**: `prettier`

Manual formatting: `./scripts/format.sh`

Skip options: `./scripts/format.sh --skip-clippy --skip-bazel`

### Conventional Commits (MANDATORY)

**All commits MUST follow conventional commit format.** The project uses:

- `commitlint` to enforce commit message format
- `release-please` for automated changelog and releases

**Required format**: `type(scope): description`

**Commit types**:

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `chore`: Maintenance (dependencies, config)
- `ci`: CI/CD changes
- `refactor`: Code refactoring
- `test`: Test additions/changes
- `perf`: Performance improvements

**Examples**:

```bash
git commit -m "feat(cli): add --verbose flag for detailed logging"
git commit -m "fix(resolver): handle cyclic dependencies correctly"
git commit -m "docs: update installation instructions"
git commit -m "chore(deps): update tokio to 1.47.1"
```

**Breaking changes**: Add `!` after type or `BREAKING CHANGE:` in footer:

```bash
git commit -m "feat(api)!: remove deprecated lock format"
```

Commits that don't follow this format will be rejected by the pre-commit hook.

## Configuration

### config.yaml Structure

```yaml
output:
  path: "./packages"
  target_architectures: ["amd64"]

source_repositories:
  - source_url: "https://snapshot.ubuntu.com/ubuntu/20260207T140000Z"
    architectures: ["amd64"]
    distributions: ["noble", "noble-updates"]

packages:
  - "package-name"
  - "package-name (= 1.2.3-4)" # Exact version pinning
```

**Repository types**:

- Standard Debian repos: `source_url` + `distributions`
- Custom paths: Use `distribution_path` for non-standard layouts (e.g., CUDA repos)
- Snapshot repos: Use timestamped URLs for reproducibility

## Testing

### Cargo Tests

```bash
# All tests
cargo test

# Verbose output
cargo test -- --nocapture

# Specific test
cargo test test_name

# E2E tests (Rust-based)
cargo test --package aptprep-e2e-tests
```

### Bazel E2E Tests (Podman-based)

**Requires Podman installed.** These tests create actual APT repositories and verify package installation in containers.

```bash
# Run Podman-based e2e test
bazel test //e2e:e2e_test

# All e2e tests
bazel test //e2e/...
```

**How it works:**

1. Runs `aptprep lock` and `download` to create local repository
2. Generates Packages index file
3. Spins up Ubuntu Podman container with local repo mounted
4. Disables online APT repositories
5. Verifies packages install from local repository only

**Note:** Tagged as "manual" - not run automatically in all CI environments.

## Gotchas

### Bazel Usage

- **DO NOT** use Bazel for development - it's slower and optimized for CI/releases
- Use Cargo for all development work (`cargo build`, `cargo test`)
- Valid Bazel use cases:
  - Release builds: `bazel build //:aptprep_linux_x86_64`
  - Podman e2e tests: `bazel test //e2e:e2e_test` (requires Podman)

### Version Pinning Workaround

- Ubuntu's packaging team sometimes uses incorrect upstream versions
- Use exact version syntax in config.yaml: `"package (= version)"` to work around this
- See branch `bugfix/workaround-ubuntu-invalid-upstream-versions` for context

### Snapshot Repositories

- Ubuntu snapshot URLs include timestamps: `snapshot.ubuntu.com/ubuntu/20260207T140000Z`
- These ensure reproducible builds but require periodic updates
- Check Ubuntu Launchpad for available snapshots when updating

### Rust Edition

- Uses Rust edition 2024 (recent, ensure toolchain is up-to-date)

## Workflow

### Development Cycle

1. Create feature branch: `git checkout -b feat/feature-name`
2. Make changes with automatic formatting via git hooks
3. Run tests: `cargo test`
4. Lint: `cargo clippy -- -D warnings`
5. Commit with conventional format: `git commit -m "feat: description"`
6. Push and create PR

### Before Creating PR

```bash
cargo test
cargo clippy -- -D warnings
cargo fmt --check
```

### Release Process

- Releases are automated via `release-please` GitHub Action
- Conventional commits drive changelog generation and version bumps
- Bazel builds create release artifacts (binary + licenses + SBOM)
