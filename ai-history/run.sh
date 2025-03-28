#!/bin/bash

cd guest; cargo build --release --target=wasm32-wasip2
clear
cd ../host; cargo build
clear
cargo run
