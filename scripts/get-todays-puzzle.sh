#!/bin/bash

# Navigate to the root directory of the project
cd "$(dirname $0)/../rust"

# Build the project in release mode
cargo build --release --package todays-puzzle

cd ..

# Check if the site/generated directory exists, and create it if it doesn't
if [ ! -d "./site/generated" ]; then
  mkdir -p "./site/generated"
fi

./rust/target/release/todays-puzzle "./site/generated/json"
