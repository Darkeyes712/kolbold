# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), 
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
### Changed
### Deprecated
### Removed

---

## [1.2.0] - 13-11-2024
### Added
- Support for measuring `swap` usage in single-threaded and multi-threaded contexts.
- kolbold-core to version `0.2.0`

### Changed
- Reworked `MemoryMeasurementData` struct to include `swap_usage` as part of memory metrics collection.

### Fixed
- Improved error handling in memory collection, especially around lock acquisition.
- Minor formatting adjustments in the logging and error messages.

