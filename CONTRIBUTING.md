# Contributing to aptprep

Thank you for your interest in contributing to aptprep! This document provides guidelines and information for contributors.

## Getting Started

### Development Setup

1. **Clone the repository**:
   ```bash
   git clone https://github.com/nickguletskii/aptprep.git
   cd aptprep
   ```

2. **Install dependencies**:
   ```bash
   # Rust toolchain (if not already installed)
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

   # Node.js dependencies for formatting
   pnpm install
   ```

3. **Set up development tools**:
   ```bash
   # Configure git to use in-repo hooks for automatic formatting
   git config core.hooksPath .githooks
   ```

4. **Verify setup**:
   ```bash
   cargo build
   cargo test

   # Optional: Test Bazel build (used for releases)
   bazel build //:aptprep_bin
   ```

### Development Environment

aptprep uses several tools for development:

- **Rust**: Main language (edition 2024)
- **Bazel**: Build system and package management (used for releases and distribution packaging)
- **pnpm**: Node.js package manager for dev tools
- **rustfmt**: Rust code formatting
- **buildifier**: Bazel file formatting
- **prettier**: Config file formatting

## Code Style

### Automatic Formatting

The project enforces consistent formatting through git hooks:

- **Rust code**: Formatted with `rustfmt` using `rustfmt.toml` configuration
- **Bazel files**: Formatted with `buildifier`
- **Config files**: Formatted with `prettier`

If you've set up git hooks, formatting happens automatically on commit.

### Manual Formatting

```bash
# Format Rust code
cargo fmt

# Format Bazel files
buildifier $(find . -name "*.bzl" -o -name "BUILD*" -o -name "MODULE.bazel")

# Format config files
pnpx prettier --write "**/*.{toml,yml,yaml,json}"
```

### Code Quality

- Follow Rust naming conventions and idioms
- Write clear, descriptive commit messages
- Add rustdoc comments for public APIs
- Use `cargo clippy` to catch common issues

## Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run with verbose output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

### Writing Tests

- Add unit tests for new functionality
- Use integration tests for end-to-end scenarios
- Test error conditions and edge cases
- Mock external dependencies when appropriate

## Pull Request Process

### Before Submitting

1. **Create a feature branch**:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes** with clear, focused commits

3. **Run tests and linting**:
   ```bash
   cargo test
   cargo clippy -- -D warnings
   cargo fmt --check
   ```

4. **Update documentation** if needed

### Pull Request Requirements

- **Description**: Clear explanation of changes and motivation
- **Tests**: Include tests for new functionality
- **Documentation**: Update docs for user-facing changes
- **Changelog**: Add entry to `CHANGELOG.md` for significant changes
- **No breaking changes** without discussion (for now, since we're pre-1.0)

### Review Process

1. Automated checks must pass (formatting, tests, linting)
2. Code review by maintainers
3. Discussion and feedback incorporation
4. Final approval and merge

## Issue Reporting

### Bug Reports

When reporting bugs, please include:

- **Environment**: OS, Rust version, aptprep version
- **Reproduction steps**: Minimal example that reproduces the issue
- **Expected behavior**: What should happen
- **Actual behavior**: What actually happens
- **Logs**: Relevant error messages or output

### Feature Requests

For new features, please describe:

- **Use case**: What problem does this solve?
- **Proposed solution**: How should it work?
- **Alternatives**: Other approaches considered
- **Breaking changes**: Impact on existing functionality

## Project Structure

```
aptprep/
├── src/                    # Main Rust source code
├── tools/licenses/         # License management tools
│   └── extract_licenses/   # Rust tool for license extraction
├── .github/               # GitHub templates and workflows
├── scripts/               # Development scripts
└── docs/                  # Additional documentation (if any)
```

### Key Components

- **CLI interface**: Command-line argument parsing and subcommands
- **Dependency resolution**: Package dependency graph resolution
- **Package downloading**: HTTP client for fetching packages
- **Bazel integration**: Generate BUILD files for rules_distroless
- **Lockfile management**: Serialization and integrity verification

### Build System

aptprep uses a dual build system approach:

#### Cargo (Development)
- **Primary development**: `cargo build`, `cargo test`, `cargo run`
- **Local testing**: Fast iteration during development
- **IDE integration**: Full Rust tooling support

#### Bazel (Releases)
- **Official releases**: All GitHub releases are built with Bazel
- **Distribution packaging**: Creates complete archives with licenses and SBOM
- **Cross-compilation**: Supports multiple architectures (x86_64, aarch64)
- **Reproducible builds**: Hermetic builds with dependency pinning

```bash
# Development workflow
cargo build
cargo test

# Release testing (matches CI)
bazel build //:aptprep_linux_x86_64
```

The Bazel build system ensures that releases are reproducible and include:
- The `aptprep` binary
- Third-party licenses (`THIRD_PARTY_LICENSES.txt`)
- Software Bill of Materials (SBOM) in CycloneDX format
- Project documentation and licenses

## Architecture Guidelines

### Error Handling

- Use `eyre` for error handling and context
- Provide meaningful error messages to users
- Include relevant context in error chains

### Logging

- Use `tracing` for structured logging
- Log at appropriate levels (error, warn, info, debug, trace)
- Include relevant context in log messages

### Dependencies

- Prefer established, well-maintained crates
- Justify new dependencies in PR descriptions
- Avoid unnecessary feature flags to minimize binary size

## Getting Help

- **Questions**: Open a GitHub issue with the "question" label
- **Discussion**: Use GitHub Discussions for broader topics
- **Security**: See `SECURITY.md` for vulnerability reporting

## Code of Conduct

This project follows the [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct). We are committed to providing a welcoming and inclusive environment for all contributors.

## License

By contributing to aptprep, you agree that your contributions will be licensed under the same license as the project (Apache-2.0 OR MIT).

Thank you for contributing!