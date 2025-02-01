#!/bin/bash

cd "$(dirname $0)/.."

npm run lint

cd rust
cargo clippy --all -- -W clippy::pedantic

