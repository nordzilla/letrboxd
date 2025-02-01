#!/bin/bash
set -euo pipefail

cd "$(dirname $0)/.."

npm run lint:fix

cd rust
cargo fmt
