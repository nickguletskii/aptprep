# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0](https://github.com/nickguletskii/aptprep/compare/v0.2.0...v0.3.0) (2026-02-11)


### Features

* add the ability to override output paths via CLI, refactor CLI handling ([33674ad](https://github.com/nickguletskii/aptprep/commit/33674ad801acf6827cc8f0018207a87411b16e53))

## [0.2.0](https://github.com/nickguletskii/aptprep/compare/v0.1.3...v0.2.0) (2026-02-11)


### Features

* Add command to generate Packages index file from lockfile ([2035e49](https://github.com/nickguletskii/aptprep/commit/2035e49e9b649da0682deb09abbcb7444e7cf154))
* CLI: use dynamic version from Cargo package metadata ([3d175bc](https://github.com/nickguletskii/aptprep/commit/3d175bc3f2e2a26bb7e1e6153c47aa8795cbaf22))


### Bug Fixes

* download and lockfile: take version constraints into account when matching packages ([f2f0d38](https://github.com/nickguletskii/aptprep/commit/f2f0d3832d841396f43d72a30ab144218e8ef634))
* download: add support for SHA384 and SHA512 checksum types ([947474d](https://github.com/nickguletskii/aptprep/commit/947474d6772c937c2fb57dbac0557064100cf69e))
* download: correct base URL construction by removing path, query, and fragment ([ea53fda](https://github.com/nickguletskii/aptprep/commit/ea53fdae750acdeb5e95d774b7591ad486f81e12))


### Documentation

* add AGENTS.md and CLAUDE.md for project documentation ([83eae37](https://github.com/nickguletskii/aptprep/commit/83eae37dfbd1ea054e538e0aa5a3d84dab8150a7))

## [0.1.3](https://github.com/nickguletskii/aptprep/compare/v0.1.2...v0.1.3) (2025-12-16)


### Bug Fixes

* Work around incorrect upstream package versions used by the Ubuntu packaging team ([be1101f](https://github.com/nickguletskii/aptprep/commit/be1101f8a510f4f0a2518427601b1593bd2555ce))

## [0.1.2](https://github.com/nickguletskii/aptprep/compare/v0.1.1...v0.1.2) (2025-11-04)


### chore

* force release ([c2bf2ba](https://github.com/nickguletskii/aptprep/commit/c2bf2bafdb2aeadc3e0b297899bba7fef1d4a115))

## [0.1.1](https://github.com/nickguletskii/aptprep/compare/v0.2.0...v0.1.1) (2025-11-04)


### chore

* release 0.1.1 ([104a7ed](https://github.com/nickguletskii/aptprep/commit/104a7edf28873172bc09734bad43b19490203cb7))
* release 0.1.1 ([82d2030](https://github.com/nickguletskii/aptprep/commit/82d203061e4c9680c48824d794858d319d6a7003))


### Features

* Initial release ([cee37ff](https://github.com/nickguletskii/aptprep/commit/cee37ff79e85630bc5ad695c2e64abc064eface7))

## [Unreleased]

## [0.2.0] - 2025-11-04

### Added
- Baseline release establishing v0.2.0 as the first stable version

[unreleased]: https://github.com/nickguletskii/aptprep/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/nickguletskii/aptprep/releases/tag/v0.2.0
