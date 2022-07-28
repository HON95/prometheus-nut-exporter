#!/bin/bash

set -eu

DC="docker-compose -f manage/docker/docker-compose.yml"

export DOCKER_BUILDKIT=1

$DC build
