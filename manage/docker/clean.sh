#!/bin/bash

set -eu

DC="docker-compose -f manage/docker/docker-compose.yml"

echo "Downing DC resources ..."
$DC down

echo
echo "Emptying Docker build cache (global) ..."
#docker image prune -af
docker builder prune -af

echo
echo "Deleting local data ..."
sudo rm -rf .local/
