#!/bin/bash

set -u

echo "Running Clippy ..."
echo
cargo clippy -- -D warnings
if (( $? != 0 )); then
    echo
    echo -e "\e[31mFailed!\e[0m" >&2
    exit 1
fi

echo
echo "Checking for trailing whitespace ..."
! egrep -RHn "\s+$" src/
if (( $? != 0 )); then
    echo
    echo -e "\e[31mFailed!\e[0m" >&2
    exit 1
fi
