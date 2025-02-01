#!/bin/bash

cd "$(dirname $0)/../rust"

rustup target add wasm32-unknown-unknown
cargo install --locked wasm-bindgen-cli

rm -rf ../site/generated/wasm

RUSTFLAGS="-C link-arg=-s" \
    cargo build --package letrboxd-wasm --release --target wasm32-unknown-unknown

wasm-bindgen ./target/wasm32-unknown-unknown/release/letrboxd.wasm --target web --out-dir ../site/generated/wasm
