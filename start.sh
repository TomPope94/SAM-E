#!/bin/bash

# Start the lambda runtime custom invoker
/app/release/sam-e-invoker &

# Start each of the sources
/app/release/sam-e-source-apigw &
/app/release/sam-e-source-sqs &
/app/release/sam-e-source-s3 &
/app/release/sam-e-source-eventbridge &

wait
