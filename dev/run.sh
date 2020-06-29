#!/bin/bash

set -eu

PROM_DATA_DIR=".local/prometheus_data"
DC="docker-compose -f dev/docker-compose.yml"

mkdir -p "$PROM_DATA_DIR"
chmod 777 "$PROM_DATA_DIR"

$DC up
