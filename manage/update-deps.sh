#!/bin/bash

IMAGE="rust:1.43"

set -eu

command="mkdir src && touch src/main.rs && cargo update"

if [[ $(uname -s) == MINGW* ]]; then
    # Double slashes to avoid MinGW Windows path conversion
    docker run --rm -v "/${PWD}/Cargo.toml://Cargo.toml:ro" -v "/${PWD}/Cargo.lock://Cargo.lock:rw" "$IMAGE" bash -c "$command"
else
    docker run --rm -v "${PWD}/Cargo.toml:/Cargo.toml:ro" -v "${PWD}/Cargo.lock:/Cargo.lock:rw" "$IMAGE" bash -c "$command"
fi
