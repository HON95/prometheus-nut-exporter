# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Added proper logging with adjustable log level.
- Added duplicate compatibility metrics to compensate for the renamed metrics in the previous release.

### Changed

- Changed request logging to use log level `debug` instead of the `LOG_REQUESTS_CONSOLE` environment variable.

### Deprecated

### Removed

### Fixed

- Fixed failing to parse non-semantic NUT versions.

### Security

## [1.1.0] - 2021-04-07

### Added

- Added UPS description to `nut_ups_info`.
- Added lots of more metrics.

### Changed

- Replaced Docker image tags `stable` and `bleeding` with `latest`, `X`, `X.Y` and `X.Y.Z` (parts of the semantic version).
- Renamed a few voltage-related metrics (slightly breaking).

### Fixed

- Added Tini as container entrypoint to handle signals properly (i.e. not stall when exiting).
- Fixed parsing error when multiple UPSes exist.

## [1.0.1] - 2020-06-29

### Added

- Added metadata metrics `nut_info` for the NUT server and `nut_exporter_info` for the exporter.

### Changed

- Improved error messages sent to client.

### Fixed

- Fixed malformed labels for `nut_ups_info`.

## [1.0.0] - 2020-06-18

Initial release.
