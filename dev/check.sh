#!/bin/bash

set -eu

cargo clippy -- -D warnings
