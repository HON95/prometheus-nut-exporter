#!/bin/bash

set -eu

DIR="$(dirname "${BASH_SOURCE[0]}")"

"$DIR/build.sh"
echo
"$DIR/run.sh"
