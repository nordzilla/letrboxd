#!/bin/bash

cd "$(dirname $0)/../rust"

cargo build --release --package letrboxd-cli

cp ./target/release/letrboxd ./../letrboxd