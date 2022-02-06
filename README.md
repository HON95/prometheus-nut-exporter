# Prometheus NUT Exporter

[![GitHub release](https://img.shields.io/github/v/release/HON95/prometheus-nut-exporter?label=Version)](https://github.com/HON95/prometheus-nut-exporter/releases)
[![CI](https://github.com/HON95/prometheus-nut-exporter/workflows/CI/badge.svg?branch=master)](https://github.com/HON95/prometheus-nut-exporter/actions?query=workflow%3ACI)
[![FOSSA status](https://app.fossa.com/api/projects/git%2Bgithub.com%2FHON95%2Fprometheus-nut-exporter.svg?type=shield)](https://app.fossa.com/projects/git%2Bgithub.com%2FHON95%2Fprometheus-nut-exporter?ref=badge_shield)
[![Docker pulls](https://img.shields.io/docker/pulls/hon95/prometheus-nut-exporter?label=Docker%20Hub)](https://hub.docker.com/r/hon95/prometheus-nut-exporter)

![Dashboard](https://grafana.com/api/dashboards/14371/images/10335/image)

A Prometheus exporter for uninterruptable power supplies (UPSes) using Network UPS Tools (NUT).

## Usage

### NUT

Set up NUT in server mode and make sure the TCP port (3493 by default) is accessible (without authentication).

If you want to test that it's working, run `telnet <nut-server> 3493` and then `VER`, `LIST UPS` and `LIST VAR <ups>`.

### Docker

Example `docker-compose.yml`:

```yaml
version: "3.7"

services:
  nut-exporter:
    image: hon95/prometheus-nut-exporter:1
    environment:
      - TZ=Europe/Oslo
      - HTTP_PATH=/metrics
      # Defaults
      #- RUST_LOG=info
      #- HTTP_PORT=9995
      #- HTTP_PATH=/nut
      #- LOG_REQUESTS_CONSOLE=false
      #- PRINT_METRICS_AND_EXIT=false
    ports:
      - "9995:9995/tcp"
```

### Prometheus

Example `prometheus.yml`:

```yaml
global:
    scrape_interval: 15s
    scrape_timeout: 10s

scrape_configs:
  - job_name: "nut"
    static_configs:
      # Insert NUT server address here
      - targets: ["nut-server:3493"]
    metrics_path: /nut
    relabel_configs:
      - source_labels: [__address__]
        target_label: __param_target
      - source_labels: [__param_target]
        target_label: instance
      - target_label: __address__
        # Insert NUT exporter address here
        replacement: nut-exporter:9995
```

In the above example, `nut-exporter:9995` is the address and port of the NUT _exporter_ while `nut-server:3493` is the address and port of the NUT _server_ to query through the exporter.

### Kubernetes Resource Usage

Example container resources requests and limits.
This was done by scraping one NUT server with two UPSes.
Resource usage was observed across 7 days period.
This can be lowered even more but should be sufficient as a starting point.

```yaml
resources:
  limits:
    cpu: "10m"
    memory: "16Mi"
  requests:
    cpu: "1m"
    memory: "8Mi"
```

### Grafana

[Example dashboard](https://grafana.com/grafana/dashboards/14371)

## Configuration

### Docker Image Versions

Use `1` for stable v1.Y.Z releases and `latest` for bleeding/unstable releases.

### Environment Variables

- `RUST_LOG` (defaults to `info`): The log level used by the console/STDOUT. Set to `debug` so show HTTP requests and `trace` to show extensive debugging info.
- `HTTP_ADDRESS` (defaults to `::`): The HTTP server will listen on this IP. Set to `127.0.0.1` or `::1` to only allow local access. 
- `HTTP_PORT` (defaults to `9995`): The HTTP server port.
- `HTTP_PATH` (defaults to `nut`): The HTTP server metrics path. You may want to set it to `/metrics` on new setups to avoid extra Prometheus configuration (not changed here due to compatibility).
- `PRINT_METRICS_AND_EXIT` (defaults to `false`): Print a Markdown-formatted table consisting of all metrics and then immediately exit. Used mainly for generating documentation.

## Metrics

| Metric | NUT Var | Unit | Description |
| - | - | - | - |
| `nut_exporter_info` |  |  | Metadata about the exporter. |
| `nut_info` |  |  | Metadata about the NUT server. |
| `nut_ups_info` |  |  | Metadata about the UPS (e.g. model, battery type, location). |
| `nut_status` | `ups.status` |  | UPS status. Unknown (0), on line (1, "OL"), on battery (2, "OB"), or low battery (3, "LB"). |
| `nut_beeper_status` | `ups.beeper.status` |  | If the beeper is enabled. Unknown (0), enabled (1), disabled (2) or muted (3). |
| `nut_uptime_seconds` | `device.uptime` | `seconds` | Device uptime. |
| `nut_load` | `ups.load` |  | Load. (0-1) |
| `nut_temperature_celsius` | `ups.temperature` | `degrees C` | UPS temperature |
| `nut_battery_charge` | `battery.charge` |  | Battery level. (0-1) |
| `nut_battery_charge_low` | `battery.charge.low` |  | Battery level threshold for low state. (0-1) |
| `nut_battery_charge_warning` | `battery.charge.warning` |  | Battery level threshold for warning state. (0-1) |
| `nut_battery_charge_restart` | `battery.charge.restart` |  | Battery level threshold for restarting after power-off. (0-1) |
| `nut_battery_runtime_seconds` | `battery.runtime` | `seconds` | Battery runtime. |
| `nut_battery_runtime_low_seconds` | `battery.runtime.low` | `seconds` | Battery runtime threshold for state low. |
| `nut_battery_runtime_restart_seconds` | `battery.runtime.restart` | `seconds` | Battery runtime threshold for restart after power-off. |
| `nut_delay_shutdown_seconds` | `ups.delay.shutdown` | `seconds` | Interval to wait after shutdown with delay command. |
| `nut_delay_start_seconds` | `ups.delay.start` | `seconds` | Interval to wait before (re)starting the load. |
| `nut_battery_voltage_volts` | `battery.voltage` | `volts` | Battery voltage. |
| `nut_battery_voltage_nominal_volts` | `battery.voltage.nominal` | `volts` | Battery voltage (nominal). |
| `nut_battery_voltage_high_volts` | `battery.voltage.high` | `volts` | Battery voltage for full (charge level calculation). |
| `nut_battery_voltage_low_volts` | `battery.voltage.low` | `volts` | Battery voltage for empty (charge level calculation). |
| `nut_battery_temperature_celsius` | `battery.temperature` | `degrees C` | Battery temperature. |
| `nut_input_voltage_volts` | `input.voltage` | `volts` | Input voltage. |
| `nut_input_voltage_nominal_volts` | `input.voltage.nominal` | `volts` | Input voltage (nominal). |
| `nut_input_voltage_minimum_volts` | `input.voltage.minimum` | `volts` | Input voltage (minimum seen). |
| `nut_input_voltage_maximum_volts` | `input.voltage.maximum` | `volts` | Input voltage (maximum seen). |
| `nut_input_transfer_low_volts` | `input.transfer.low` | `volts` | Input lower transfer threshold. |
| `nut_input_transfer_high_volts` | `input.transfer.high` | `volts` | Input upper transfer threshold. |
| `nut_input_current_amperes` | `input.current` | `amperes` | Input current. |
| `nut_input_current_nominal_amperes` | `input.current.nominal` | `amperes` | Input current (nominal). |
| `nut_input_frequency_hertz` | `input.frequency` | `hertz` | Input frequency. |
| `nut_input_frequency_nominal_hertz` | `input.frequency.nominal` | `hertz` | Input frequency (nominal). |
| `nut_input_frequency_low_hertz` | `input.frequency.low` | `hertz` | Input frequency (low). |
| `nut_input_frequency_high_hertz` | `input.frequency.high` | `hertz` | Input frequency (high). |
| `nut_output_voltage_volts` | `output.voltage` | `volts` | Output voltage. |
| `nut_output_voltage_nominal_volts` | `output.voltage.nominal` | `volts` | Output voltage (nominal). |
| `nut_output_current_amperes` | `output.current` | `amperes` | Output current. |
| `nut_output_current_nominal_amperes` | `output.current.nominal` | `amperes` | Output current (nominal). |
| `nut_output_frequency_hertz` | `output.frequency` | `hertz` | Output frequency. |
| `nut_output_frequency_nominal_hertz` | `output.frequency.nominal` | `hertz` | Output frequency (nominal). |
| `nut_power_watts` | `ups.power` | `watts` | Apparent power. |
| `nut_power_nominal_watts` | `ups.power.nominal` | `watts` | Apparent power (nominal). |
| `nut_real_power_watts` | `ups.realpower` | `watts` | Real power. |
| `nut_real_power_nominal_watts` | `ups.realpower.nominal` | `watts` | Real power (nominal). |
| `nut_battery_volts` | `battery.voltage` | `volts` | Battery voltage. (Compatibility metric, use nut_battery_voltage_volts instead.) |
| `nut_input_volts` | `input.voltage` | `volts` | Input voltage. (Compatibility metric, use nut_input_voltage_volts instead.) |
| `nut_output_volts` | `output.voltage` | `volts` | Output voltage. (Compatibility metric, use nut_output_voltage_volts instead.) |


(Generated by setting `PRINT_METRICS_AND_EXIT=true`.)

Feel free to suggest adding more metrics (including a printout of what the variable and value looks like for your UPS)!

To check if a specific UPS is unavailable, use something like: `absent(nut_status{job="...", ups="..."})`

## References

- [NUT: GENERICUPS(8)](https://networkupstools.org/docs/man/genericups.html)
- [NUT Developer Guide: 9. Network protocol information](https://networkupstools.org/docs/developer-guide.chunked/ar01s09.html)
- [NUT Developer Guide: A.1. Variables](https://networkupstools.org/docs/developer-guide.chunked/apas01.html)

## License

GNU General Public License version 3 (GPLv3).
