#!/bin/bash
set -euo pipefail

cd "$(dirname $0)/.."

npm audit fix

cd rust
cargo update --recursive
