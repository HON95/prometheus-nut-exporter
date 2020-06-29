#!/bin/bash

set -eu

DC="docker-compose -f dev/docker-compose.yml"

$DC build
