#!/bin/bash

set -eu

DC="docker-compose -f manage/docker-compose.yml"

echo "Downing DC resources ..."
$DC down

echo
echo "Emptying Docker build cache ..."
docker builder prune -af

echo
echo "Deleting build files ..."
rm -rf .target/

echo
echo "Deleting local data ..."
sudo rm -rf .local/
