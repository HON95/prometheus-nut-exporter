#!/bin/bash

IMAGE="prometheus-nut-exporter:dev"
NAME="prometheus-nut-exporter-dev"
PORT="9999"

set -eu

command="docker run --rm -ti -p $PORT:$PORT --name=$NAME $IMAGE"

if [[ $(uname -s) = MINGW* ]]; then
    winpty $command
else
    $command
fi
