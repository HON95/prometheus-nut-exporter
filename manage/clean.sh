#!/bin/bash

set -eu

echo "Emptying Docker build cache ..."
docker builder prune -af
