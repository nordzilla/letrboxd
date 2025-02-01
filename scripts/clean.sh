#!/bin/bash
set -euo pipefail

cd "$(dirname $0)/.."

rm -rf letrboxd

cd rust
cargo clean
