#!/bin/bash

cd guest-gpt; cargo build --release --target=wasm32-wasip2
cd ../host-gpt; cargo run