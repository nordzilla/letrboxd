#!/bin/bash
set -euo pipefail

cd "$(dirname $0)/.."

npm run lint

cd rust
cargo fmt --check
cargo clippy --all -- -W clippy::pedantic
