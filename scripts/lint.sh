#!/bin/bash

cd "$(dirname $0)/.."

cd rust
cargo clippy --all -- -W clippy::pedantic

