# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.0](https://github.com/so1ve/injectium/compare/injectium-v0.3.0...injectium-v0.4.0) - 2026-03-06

### Added

- *(core)* [**breaking**] unify provider model and tighten container registration semantics

## [0.2.1](https://github.com/so1ve/injectium/compare/injectium-v0.2.0...injectium-v0.2.1) - 2026-03-03

### Added

- move validation logic behind `validation` feature gate

### Other

- *(core)* switch container runtime storage from HashMap to packed slices
- *(core)* add optional ahash map backend for TypeId storage
- remove Injectable derive from Db struct
- remove Injectable derive from Db struct

## [0.2.0](https://github.com/so1ve/injectium/compare/injectium-v0.1.1...injectium-v0.2.0) - 2026-02-24

### Added

- [**breaking**] move `declare_dependency` macro to core

## [0.1.1](https://github.com/so1ve/injectium/compare/injectium-v0.1.0...injectium-v0.1.1) - 2026-02-24

### Other

- add proper documentation for crates
