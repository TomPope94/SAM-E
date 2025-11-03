#!/bin/bash

# Test script for Lambda streaming functionality
# This script demonstrates how to test both streaming and non-streaming endpoints

set -e

BLUE='\033[0;34m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}Testing Lambda Streaming Functionality${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Test 1: Non-streaming endpoint
echo -e "${GREEN}Test 1: Non-Streaming Response${NC}"
echo -e "${YELLOW}Testing /normal endpoint without streaming header...${NC}"
echo ""
curl -s http://localhost:3000/normal | jq .
echo ""
echo ""

# Test 2: Streaming endpoint with header
echo -e "${GREEN}Test 2: Streaming Response${NC}"
echo -e "${YELLOW}Testing /stream endpoint WITH streaming header...${NC}"
echo -e "${YELLOW}(Watch for progressive output)${NC}"
echo ""
curl -H "X-Amz-Invocation-Type: ResponseStream" http://localhost:3000/stream
echo ""
echo ""

# Test 3: Chunks endpoint with streaming
echo -e "${GREEN}Test 3: Multiple Chunks with Streaming${NC}"
echo -e "${YELLOW}Testing /chunks endpoint WITH streaming header...${NC}"
echo ""
curl -H "X-Amz-Invocation-Type: ResponseStream" http://localhost:3000/chunks | jq .
echo ""
echo ""

# Test 4: Same endpoint without streaming (for comparison)
echo -e "${GREEN}Test 4: Same Endpoint Without Streaming${NC}"
echo -e "${YELLOW}Testing /chunks endpoint WITHOUT streaming header...${NC}"
echo ""
curl -s http://localhost:3000/chunks | jq .
echo ""
echo ""

# Test 5: Health check
echo -e "${GREEN}Test 5: Health Check${NC}"
echo -e "${YELLOW}Testing /health endpoint...${NC}"
echo ""
curl -s http://localhost:3000/health
echo ""
echo ""

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}All tests completed!${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""
echo -e "${YELLOW}Tips:${NC}"
echo -e "  • Compare the response times between streaming and non-streaming"
echo -e "  • Watch the SAM-E logs to see chunk handling"
echo -e "  • Use 'curl -v' to see HTTP headers in detail"
echo -e "  • Check for 'Transfer-Encoding: chunked' in streaming responses"
echo ""
