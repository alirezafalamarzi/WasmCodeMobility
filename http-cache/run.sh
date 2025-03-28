#!/bin/bash

cd guest-cache; cargo build --release --target=wasm32-wasip2
cd ../host; cargo run
