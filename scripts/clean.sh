#!/bin/bash

cd "$(dirname $0)/.."

rm -rf letrboxd

cd rust
cargo clean

