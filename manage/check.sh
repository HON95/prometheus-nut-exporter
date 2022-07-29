#!/bin/bash

set -u

src_dir="src/"

function fail {
    echo
    echo -e "\e[31mFailed!\e[0m" >&2
    exit 1
}

function check_fail {
    if (( $? != 0 )); then
        fail
    fi
}

echo "Running Clippy ..."
clippy_args="-D warnings \
-A clippy::branches-sharing-code \
-A clippy::vec-init-then-push"
cargo clippy -- $clippy_args
check_fail

echo
echo "Checking for trailing whitespace ..."
! egrep -RHn "\s+$" "$src_dir"
check_fail

echo
echo "Checking for multiple empty lines ..."
for file in $(find "$src_dir" -type f); do
    line_number=0
    empty_lines=0
    while IFS= read -r line; do
        ((line_number++))
        if [[ $line == "" ]]; then
            ((empty_lines++))
        else
            empty_lines=0
        fi
        if (( empty_lines > 1 )); then
            echo "Bad file: $file [line $line_number]" >&2
            fail
        fi
    done <"$file"
done

echo
echo "Checking for empty line at end of files ..."
for file in $(find "$src_dir" -type f); do
    if [[ $(tail -c1 "$file") != "" ]]; then
        echo "Bad file: $file" >&2
        fail
    fi
done

echo
echo -e "\e[32mSuccess!\e[0m"
