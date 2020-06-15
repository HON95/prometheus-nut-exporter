#!/bin/bash

set -eu

DC="docker-compose -f manage/docker-compose.yml"

$DC build

$DC up
