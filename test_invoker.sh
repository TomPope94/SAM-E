#!/usr/bin/env bash

# Start the lambda runtime custom invoker
# docker run \
#   -it \
#   -e CONFIG="$(cat ./test_invoker_config.yaml)" \
#   -e RUST_LOG=debug \
#   -p 3000:3000 \
#   -p 3001:3001 \
#   -p 3002:3002 \
#   sam-e-invoker 

CONFIG="$(cat ./test_invoker_config.yaml)" docker-compose up --build
