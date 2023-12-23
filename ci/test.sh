#!/bin/sh
# MIT/Apache2 License

set -eu

cd "$(dirname -- "$(dirname -- "$0")")" || exit 1
cargo test
