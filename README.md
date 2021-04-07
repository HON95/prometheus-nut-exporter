# Prometheus NUT Exporter

[![GitHub release](https://img.shields.io/github/v/release/HON95/prometheus-nut-exporter?label=Version)](https://github.com/HON95/prometheus-nut-exporter/releases)
[![CI](https://github.com/HON95/prometheus-nut-exporter/workflows/CI/badge.svg?branch=master)](https://github.com/HON95/prometheus-nut-exporter/actions?query=workflow%3ACI)
[![FOSSA status](https://app.fossa.com/api/projects/git%2Bgithub.com%2FHON95%2Fprometheus-nut-exporter.svg?type=shield)](https://app.fossa.com/projects/git%2Bgithub.com%2FHON95%2Fprometheus-nut-exporter?ref=badge_shield)
[![Docker pulls](https://img.shields.io/docker/pulls/hon95/prometheus-nut-exporter?label=Docker%20Hub)](https://hub.docker.com/r/hon95/prometheus-nut-exporter)

A Prometheus exporter for uninterruptable power supplies (UPSes) using Network UPS Tools (NUT).

## Usage

### NUT

Set up NUT in server mode and make sure the TCP port (3493 by default) is accessible.

### Docker

**Note**: The Docker image tags `stable` and `bleeding` are no longer used. Use `latest` (unstable) and `1` (stable v1) instead.

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

See [metrics](metrics.md).

To check if a specific UPS is unavailable, use something like: `absent(nut_status{job="...", ups="..."})`

I only have a few PowerWalker UPSes to test with, so I've only added metrics for useful variables for those. If you want more metrics/vars, you're welcome to request it or implement it yourself.

## References

- [NUT: GENERICUPS(8)](https://networkupstools.org/docs/man/genericups.html)
- [NUT Developer Guide: 9. Network protocol information](https://networkupstools.org/docs/developer-guide.chunked/ar01s09.html)
- [NUT Developer Guide: A.1. Variables](https://networkupstools.org/docs/developer-guide.chunked/apas01.html)

## License

GNU General Public License version 3 (GPLv3).
