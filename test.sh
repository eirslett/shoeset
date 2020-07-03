#!/bin/bash
cargo test
wasm-pack test --node
# wasm-pack test --chrome
