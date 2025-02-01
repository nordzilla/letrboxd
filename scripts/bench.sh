#!/bin/bash
set -euo pipefail

cd "$(dirname $0)/.."

cd rust
cargo bench
