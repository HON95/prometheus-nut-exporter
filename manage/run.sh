#!/bin/bash

set -eu

PROM_DATA_DIR=".local/prometheus_data"
DC="docker-compose -f manage/docker-compose.yml"

# Add Prometheus data dir with correct permissions
mkdir -p "$PROM_DATA_DIR"
chmod 777 "$PROM_DATA_DIR"

$DC up
