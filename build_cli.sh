#!/bin/bash

# Build the CLI
cd sam-e-cli && cargo zigbuild --release --target x86_64-unknown-linux-gnu && cargo zigbuild --release --target aarch64-unknown-linux-gnu;

# Detect architecture and copy appropriate binary
if [ "$(uname -m)" = "aarch64" ]; then
    sudo cp target/aarch64-unknown-linux-gnu/release/sam-e-cli /usr/local/bin/sam-e
else
    sudo cp target/x86_64-unknown-linux-gnu/release/sam-e-cli /usr/local/bin/sam-e
fi
