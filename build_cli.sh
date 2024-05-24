#!/bin/bash

# Build the CLI
cargo build --path sam-e-cli --release && sudo cp ./sam-e-cli/target/release/sam-e-cli /usr/local/bin/sam-e


