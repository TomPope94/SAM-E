#!/bin/bash

# Build the CLI
cd sam-e-cli && cargo build --release && sudo cp target/release/sam-e-cli /usr/local/bin/sam-e

# # Detect architecture and copy appropriate binary
# if [ "$(uname -m)" = "aarch64" ]; then
#     sudo cp target/aarch64-unknown-linux-gnu/release/sam-e-cli /usr/local/bin/sam-e
# else
#     sudo cp target/x86_64-unknown-linux-gnu/release/sam-e-cli /usr/local/bin/sam-e
# fi
