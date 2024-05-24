#!/bin/bash

# Build the CLI
cd sam-e-cli && cargo build --release && sudo cp target/release/sam-e-cli /usr/local/bin/sam-e


