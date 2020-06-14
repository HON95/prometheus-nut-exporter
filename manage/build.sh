#!/bin/bash

IMAGE="prometheus-nut-exporter:dev"
APP_ENV=${APP_ENV:-"dev"}

set -eu

DOCKER_BUILDKIT=1 docker build --build-arg="APP_ENV=$APP_ENV" -t "$IMAGE" .
