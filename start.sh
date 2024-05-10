#!/bin/bash

# Start the first process
/app/release/sam-e-invoker &

# Start the second process
/app/release/sam-e-source-apigw &

wait
