# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0](https://github.com/so1ve/injectium/compare/injectium-core-v0.2.1...injectium-core-v0.3.0) - 2026-03-03

### Added

- *(core)* [**breaking**] use container with capacity in `container!` macro output automatically, remove `capacity` field

## [0.2.1](https://github.com/so1ve/injectium/compare/injectium-core-v0.2.0...injectium-core-v0.2.1) - 2026-03-03

### Added

- move validation logic behind `validation` feature gate

### Other

- *(core)* switch container runtime storage from HashMap to packed slices
- *(core)* refactor helper functions and make clippy happy
- apply automatic fixes
- *(bench)* add resolve benchmarks and cargo aliases
- *(core)* add optional ahash map backend for TypeId storage
- *(core)* add capacity-aware container builder and macro variants
- *(core)* remove runtime downcast checks in container hot path

## [0.2.0](https://github.com/so1ve/injectium/compare/injectium-core-v0.1.1...injectium-core-v0.2.0) - 2026-02-24

### Added

- [**breaking**] move `declare_dependency` macro to core

### Other

- update license section with author name

## [0.1.1](https://github.com/so1ve/injectium/compare/injectium-core-v0.1.0...injectium-core-v0.1.1) - 2026-02-24

### Other

- add proper documentation for crates
