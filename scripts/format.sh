#!/bin/bash

cd "$(dirname $0)/.."

npm run lint:fix

cd rust
cargo fmt

