#!/bin/bash

cd "$(dirname $0)/.."

cd rust
cargo update --recursive

