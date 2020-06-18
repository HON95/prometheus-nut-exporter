# Prometheus NUT Exporter

![Rust](https://github.com/HON95/prometheus-nut-exporter/workflows/CI/badge.svg?branch=master)

A Prometheus exporter for uninterruptable power supplies (UPSes) using Network UPS Tools (NUT).

## Usage

### NUT

**TODO**

### Docker

**TODO**

### Prometheus

**TODO**

## Metrics

See [metrics](metrics.md).

I only have a few PowerWalker UPSes to test with, so I've only added matrics for useful variables for those. If you want more metrics/vars, you're welcome to request it or implement it yourself.

## References

- [NUT: GENERICUPS(8)](https://networkupstools.org/docs/man/genericups.html)
- [NUT Developer Guide: 9. Network protocol information](https://networkupstools.org/docs/developer-guide.chunked/ar01s09.html)
- [NUT Developer Guide: A.1. Variables](https://networkupstools.org/docs/developer-guide.chunked/apas01.html)
- [Prometheus: Writing exporters](https://prometheus.io/docs/instrumenting/writing_exporters/)

## License

GNU General Public License version 3 (GPLv3).
