#!/bin/bash

set -eu

DC="docker-compose -f manage/docker-compose.yml"

# Add placeholder version file
echo "0.0.0-SNAPSHOT" > VERSION

$DC build
