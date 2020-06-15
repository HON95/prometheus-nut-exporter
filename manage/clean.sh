#!/bin/bash

set -eu

DC="docker-compose -f manage/docker-compose.yml"

echo "Downing DC container ..."
$DC down

echo
echo "Emptying Docker build cache ..."
docker builder prune -af
