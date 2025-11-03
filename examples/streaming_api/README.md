# Streaming API Example

This example demonstrates how to use Lambda response streaming with SAM-E. Response streaming allows your Lambda function to send data back to the client progressively, as it becomes available, rather than buffering the entire response before sending.

## Examples in This Folder

This folder contains two different Lambda examples:

1. **`streaming-lambda/`** - Rust Lambda demonstrating HTTP response streaming
   - Shows how SAM-E handles progressive response delivery
   - Auto-completion for non-streaming Lambda runtimes
   - Multiple test endpoints for different patterns

2. **`fastmcp-handler/`** - Python Lambda using FastMCP (Model Context Protocol) with streaming
   - FastMCP framework for building MCP servers
   - Lambda Web Adapter for running FastAPI on Lambda
   - SSE (Server-Sent Events) streaming support
   - 8 callable tools for LLM integration
   - Stateless HTTP mode for serverless

See each folder's README for specific usage instructions.

## Benefits of Streaming

- **Lower latency**: Start sending data immediately without waiting for the entire response
- **Memory efficiency**: Don't need to buffer large responses in memory
- **Better user experience**: Progressive loading for large datasets
- **Real-time processing**: Stream processing results as they're generated

## Example Endpoints

This example includes several endpoints that demonstrate different response patterns:

1. **`/normal`** - Standard non-streaming response (complete response at once)
2. **`/stream`** - Streaming response example
3. **`/chunks`** - Multiple chunks generated over time
4. **`/health`** - Simple health check

## How Streaming Works in SAM-E

### Architecture

```
Client (with streaming header)
    ↓
API Gateway Source
    ↓ (forwards X-Amz-Invocation-Type: ResponseStream header)
Invoker (/invoke endpoint)
    ↓ (creates mpsc channel)
Lambda Runtime API
    ↓ (Lambda polls /next)
Lambda Function
    ↓ (posts chunks to /response)
Response Handler
    ↓ (sends chunks through channel)
Stream
    ↓
Client (receives chunks progressively)
```

### Key Components

1. **Client Request**: Include `X-Amz-Invocation-Type: ResponseStream` header
2. **Channel Setup**: Invoker creates an unbounded channel for streaming
3. **Chunk Transmission**: Lambda posts chunks with response data
4. **Completion Signal**: Final chunk includes `x-lambda-stream-complete: true` header
5. **Stream Closure**: Channel closes, completing the HTTP response

## Testing the Example

### 1. Start SAM-E

From the root of the SAM-E project:

```bash
# Build the example Lambda (if not already built)
cd examples/streaming_api/streaming-lambda
docker build -t streaming-lambda .
cd ../../..

# Start SAM-E with the streaming example
sam-e start --template examples/streaming_api/template.yaml
```

### 2. Test Non-Streaming Endpoint

Regular request without streaming:

```bash
curl http://localhost:3000/normal
```

Expected output:
```json
{
  "message": "This is a normal, non-streaming response",
  "type": "complete",
  "timestamp": "2024-10-28T..."
}
```

### 3. Test Streaming Endpoint

Request with streaming enabled:

```bash
curl -H "X-Amz-Invocation-Type: ResponseStream" \
     http://localhost:3000/stream
```

Expected output (progressively received):
```
Chunk 1: Generated at 2024-10-28T...
Chunk 2: Generated at 2024-10-28T...
Chunk 3: Generated at 2024-10-28T...
Chunk 4: Generated at 2024-10-28T...
Chunk 5: Generated at 2024-10-28T...
```

### 4. Test Chunks Endpoint

See progressive data generation:

```bash
curl -H "X-Amz-Invocation-Type: ResponseStream" \
     http://localhost:3000/chunks
```

### 5. Compare Response Times

Compare the difference between streaming and buffered:

```bash
# Non-streaming (waits for complete response)
time curl http://localhost:3000/chunks

# Streaming (starts receiving data immediately)
time curl -H "X-Amz-Invocation-Type: ResponseStream" \
     http://localhost:3000/chunks
```

## Implementing Streaming in Your Lambda

To implement streaming in your own Lambda functions, the Lambda Runtime API handles most of the complexity. Here's what you need to know:

### Lambda Runtime Perspective

When your Lambda is invoked with streaming:

1. **Poll for invocation**: Same as normal - call `/next` endpoint
2. **Process request**: Generate your response data
3. **Send chunks**: Post chunks to `/response` endpoint as they're ready
4. **Signal completion**: Include `x-lambda-stream-complete: true` in final response headers

### Example Lambda Runtime API Usage

```rust
// This is what the Lambda runtime does internally:

// 1. Get the invocation
let invocation = runtime.next().await?;

// 2. Process and stream chunks
for chunk in generate_data() {
    runtime.post_chunk(invocation.id, chunk).await?;
}

// 3. Signal completion
runtime.complete(invocation.id).await?;
```

### Current Implementation Note

The example Lambda in this directory demonstrates the concept but uses the standard `lambda_http` runtime, which doesn't yet have native streaming support. To fully test streaming:

1. The Lambda would need to manually interact with the Runtime API endpoints
2. Post chunks incrementally to `/runtime/invocation/{id}/response`
3. Set `x-lambda-stream-complete: true` on the final chunk

## Architecture Details

### Components Modified for Streaming

1. **`sam-e-types/src/invocation.rs`**
   - Added `StreamChunk` enum for chunk types
   - Added `Streaming` status
   - Added `stream_sender` channel field

2. **`sam-e-invoker/src/invocation/response.rs`**
   - Detects streaming invocations
   - Forwards chunks through the channel
   - Handles completion signals

3. **`sam-e-invoker/src/invocation.rs`**
   - Detects `X-Amz-Invocation-Type: ResponseStream` header
   - Creates mpsc channel for streaming
   - Returns chunked HTTP response

4. **`sources/sam-e-source-apigw/src/request.rs`**
   - Forwards streaming header to invoker
   - Detects streaming responses
   - Streams chunks back to client

### Data Flow

```
StreamChunk::Data(Vec<u8>)   → Sent through channel → HTTP chunk to client
StreamChunk::Complete         → Closes channel → HTTP stream ends
StreamChunk::Error(String)    → Error through channel → HTTP error
```

## Limitations & Future Work

### Current Limitations

1. Standard `lambda_http` runtime doesn't expose streaming API yet
2. No EventStream protocol support (AWS's binary framing format)
3. No CRC validation
4. Limited error recovery mid-stream

### Future Enhancements

- [ ] Implement AWS EventStream wire protocol
- [ ] Add CRC32 checksum validation
- [ ] Better error handling for partial streams
- [ ] Configurable buffer sizes and backpressure
- [ ] Metrics for streaming performance
- [ ] Support for other sources (SQS, S3)

## Troubleshooting

### "No streaming response detected"

Make sure you're including the header:
```bash
-H "X-Amz-Invocation-Type: ResponseStream"
```

### "Connection closed prematurely"

Check Lambda logs for errors. The Lambda might have crashed mid-stream.

### "Response buffered instead of streamed"

Verify the content-type in the response includes `vnd.awslambda`.

## References

- [AWS Lambda Response Streaming](https://docs.aws.amazon.com/lambda/latest/dg/configuration-response-streaming.html)
- [Lambda Runtime API](https://docs.aws.amazon.com/lambda/latest/dg/runtimes-api.html)
- [EventStream Protocol](https://docs.aws.amazon.com/AmazonS3/latest/API/RESTObjectSELECTContent.html#RESTObjectSELECTContent-responses)
