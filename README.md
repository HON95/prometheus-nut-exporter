# Prometheus NUT Exporter

[![GitHub release](https://img.shields.io/github/v/release/HON95/prometheus-nut-exporter?label=Version)](https://github.com/HON95/prometheus-nut-exporter/releases)
[![CI](https://github.com/HON95/prometheus-nut-exporter/workflows/CI/badge.svg?branch=master)](https://github.com/HON95/prometheus-nut-exporter/actions?query=workflow%3ACI)
[![FOSSA status](https://app.fossa.com/api/projects/git%2Bgithub.com%2FHON95%2Fprometheus-nut-exporter.svg?type=shield)](https://app.fossa.com/projects/git%2Bgithub.com%2FHON95%2Fprometheus-nut-exporter?ref=badge_shield)
[![Docker pulls](https://img.shields.io/docker/pulls/hon95/prometheus-nut-exporter?label=Docker%20Hub)](https://hub.docker.com/r/hon95/prometheus-nut-exporter)

A Prometheus exporter for uninterruptable power supplies (UPSes) using Network UPS Tools (NUT).

**Note**: The Docker image tags `stable` and `bleeding` are no longer used. Use `1` (stable v1) and `latest` (unstable) instead.

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
      # Defaults
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

## Metrics

| Metric | NUT Var | Unit | Description |
| - | - | - | - |
| `nut_exporter_info` |  |  | Metadata about the exporter. |
| `nut_info` |  |  | Metadata about the NUT server. |
| `nut_ups_info` |  |  | Metadata about the UPS (e.g. model, battery type, location). |
| `nut_status` | `ups.status` |  | UPS status. Unknown (0), on line (1, "OL"), on battery (2, "OB"), or low battery (3, "LB"). |
| `nut_beeper_status` | `ups.beeper.status` |  | If the beeper is enabled. Unknown (0), enabled (1), disabled (2) or muted (3). |
| `nut_uptime_seconds` | `device.uptime` | `s` | Device uptime. |
| `nut_load` | `ups.load` |  | Load. (0-1) |
| `nut_battery_charge` | `battery.charge` |  | Battery level. (0-1) |
| `nut_battery_charge_low` | `battery.charge.low` |  | Battery level threshold for low state. (0-1) |
| `nut_battery_charge_warning` | `battery.charge.warning` |  | Battery level threshold for warning state. (0-1) |
| `nut_battery_charge_restart` | `battery.charge.restart` |  | Battery level threshold for restarting after power-off. (0-1) |
| `nut_battery_runtime_seconds` | `battery.runtime` | `s` | Battery runtime. |
| `nut_battery_runtime_low_seconds` | `battery.runtime.low` | `s` | Battery runtime threshold for state low. |
| `nut_battery_runtime_restart_seconds` | `battery.runtime.restart` | `s` | Battery runtime threshold for restart after power-off. |
| `nut_battery_voltage_volts` | `battery.voltage` | `V` | Battery voltage. |
| `nut_battery_voltage_nominal_volts` | `battery.voltage.nominal` | `V` | Battery voltage (nominal). |
| `nut_battery_temperature_celsius` | `battery.temperature` | `degrees C` | Battery temperature. |
| `nut_input_voltage_volts` | `input.voltage` | `V` | Input voltage. |
| `nut_input_voltage_nominal_volts` | `input.voltage.nominal` | `V` | Input voltage (nominal). |
| `nut_input_voltage_minimum_volts` | `input.voltage.minimum` | `V` | Input voltage (minimum seen). |
| `nut_input_voltage_maximum_volts` | `input.voltage.maximum` | `V` | Input voltage (maximum seen). |
| `nut_input_transfer_low_volts` | `input.transfer.low` | `V` | Input lower transfer threshold. |
| `nut_input_transfer_high_volts` | `input.transfer.high` | `V` | Input upper transfer threshold. |
| `nut_input_current_amperes` | `input.current` | `A` | Input current. |
| `nut_input_current_nominal_amperes` | `input.current.nominal` | `A` | Input current (nominal). |
| `nut_input_frequency_hertz` | `input.frequency` | `Hz` | Input frequency. |
| `nut_input_frequency_nominal_hertz` | `input.frequency.nominal` | `Hz` | Input frequency (nominal). |
| `nut_input_frequency_low_hertz` | `input.frequency.low` | `Hz` | Input frequency (low). |
| `nut_input_frequency_high_hertz` | `input.frequency.high` | `Hz` | Input frequency (high). |
| `nut_output_voltage_volts` | `output.voltage` | `V` | Output voltage. |
| `nut_output_voltage_nominal_volts` | `output.voltage.nominal` | `V` | Output voltage (nominal). |
| `nut_output_current_amperes` | `output.current` | `A` | Output current. |
| `nut_output_current_nominal_amperes` | `output.current.nominal` | `A` | Output current (nominal). |
| `nut_output_frequency_hertz` | `output.frequency` | `Hz` | Output frequency. |
| `nut_output_frequency_nominal_hertz` | `output.frequency.nominal` | `Hz` | Output frequency (nominal). |
| `nut_power_watts` | `ups.power` | `W` | Apparent power. |
| `nut_power_nominal_watts` | `ups.power.nominal` | `W` | Apparent power (nominal). |
| `nut_real_power_watts` | `ups.realpower` | `W` | Real power. |
| `nut_real_power_nominal_watts` | `ups.realpower.nominal` | `W` | Real power (nominal). |

(Generated by setting `PRINT_METRICS_AND_EXIT=true`.)

Feel free to suggest adding more metrics (including a printout of what the variable and value looks like for your UPS)!

To check if a specific UPS is unavailable, use something like: `absent(nut_status{job="...", ups="..."})`

## References

- [NUT: GENERICUPS(8)](https://networkupstools.org/docs/man/genericups.html)
- [NUT Developer Guide: 9. Network protocol information](https://networkupstools.org/docs/developer-guide.chunked/ar01s09.html)
- [NUT Developer Guide: A.1. Variables](https://networkupstools.org/docs/developer-guide.chunked/apas01.html)

## License

GNU General Public License version 3 (GPLv3).
