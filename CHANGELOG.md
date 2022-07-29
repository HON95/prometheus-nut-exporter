# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Added metrics (@uncleBorsch):
    - `nut_temperature_celsius` (`ups.temperature`)
    - `nut_delay_shutdown_seconds` (`ups.delay.shutdown`)
    - `nut_delay_start_seconds` (`ups.delay.start`)
    - `nut_battery_voltage_high_volts` (`battery.voltage.high`)
    - `nut_battery_voltage_low_volts` (`battery.voltage.low`)
- Added more UPS metadata (@uncleBorsch).
- Added support for binding to a specific IP address through the `HTTP_ADDRESS` environment variable (@nsapa).
- Added metric `nut_ups_status` as a state set with support for many UPS statuses (`OL`, `OB`, `LB`, `CHRG` etc.).
- Added labels `driver_version`, `driver_version_internal`, `driver_version_data` and `manufacturing_date` to the `nut_ups_info` metric.
- Added proper signal handling to shutdown gracefully and not hang.
- Added multi-architecture support (Docker images for different architectures will get published to Docker Hub).

### Changed

- Changed default log level to `info`.
- Made the target port default to 3493 instead of requiring one to be provided.
- Made the Prometheus/OpenMetrics output (more) OpenMetrics 1.0.0-compliant.

### Deprecated

- Deprecated `nut_info` and added `nut_server_info` as an identical but better named replacement.
- Deprecated `nut_status` as it has very limited support for UPS statuses and doesn't always work as intended. See `nut_ups_status` for the replacement.
- Deprecated the `type` and `nut_version` labels from the `nut_ups_info` metric. Use `device_type` and `driver_version` instead.
- Deprecated `nut_battery_volts`, `nut_input_volts` and `nut_output_volts`. Replacements were added in v1.1.0.

### Removed

- Removed the tini init system from the Docker image, since signals are handled properly now.

### Fixed

- Fixed typo in `device.mfr` variable (@nsapa).

### Security

## [1.1.1] - 2021-04-11

### Added

- Added proper logging with adjustable log level.
- Added duplicate compatibility metrics to compensate for the renamed metrics in the previous release.

### Changed

- Changed request logging to use log level `debug` instead of the `LOG_REQUESTS_CONSOLE` environment variable.

### Fixed

- Fixed failing to parse non-semantic NUT versions.

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
