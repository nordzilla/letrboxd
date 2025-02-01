#!/bin/bash

cd "$(dirname $0)/.."

npm audit fix

cd rust
cargo update --recursive
