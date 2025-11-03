# SAM-E Streaming Support - Executive Summary

## Quick Overview

SAM-E is a local serverless invocation framework that simulates AWS Lambda execution. It currently uses a **synchronous request-response pattern** where:

1. External sources (API Gateway, SQS, EventBridge) POST invocation requests to the invoker
2. The invoker stores the invocation and waits for Lambda execution to complete
3. Once Lambda completes, the entire response is returned to the source
4. The source returns the response to the original client

**Key insight**: The entire flow is **blocking** - the source adapter hangs waiting for complete Lambda execution before responding to clients.

---

## Current Architecture (High Level)

```
Client Request
    ↓
Source Adapter (port 3000/3001/3002)
    ↓ POST /invoke (blocks waiting)
Invoker (port 3030)
    ├─ Store invocation in memory queue
    └─ Poll for Lambda completion (every 100ms)
        ↓
Lambda Container
    ├─ GET /next (waits for event)
    ├─ Execute handler
    └─ POST /response (return result)
        ↓
Invoker receives complete response
    ↓
Source adapter unblocks, returns response to client
```

**Key bottleneck**: Source adapter blocks waiting for Lambda to finish execution. During long-running functions, the connection from client to source remains open but idle.

---

## Why InvokeWithResponseStream Matters

AWS Lambda's `InvokeWithResponseStream` API (released 2024) allows:
- **Incremental response streaming**: Lambda can send response data in chunks
- **Real-time delivery**: Client receives data as it's generated
- **Improved latency**: No need to wait for complete execution
- **Better resource utilization**: Don't hold idle connections

### Use Cases
- Large file processing (streaming chunks to client)
- Real-time data processing (stream results as computed)
- Long-running operations (progressive updates)
- Machine learning inference (stream results incrementally)

---

## Current Non-Streaming Architecture

### Data Flow
```
Source → /invoke {Invocation}
           ↓
         write_invocation_to_store()
         Invocation { status: Pending }
           ↓
         read_invocation_from_store()
         Poll every 100ms waiting for status = Processed
           ↓
         Lambda /next gets event
         Lambda executes
         Lambda /response sends complete response
         Status changes to Processed
           ↓
         invoke() unblocks
         Returns complete ApiGatewayProxyResponse
           ↓
         Source returns to client
```

### Core Structures

**Invocation** (sam-e-types/src/invocation.rs):
```rust
pub struct Invocation {
    request_id: Uuid,
    status: Status,  // Pending, Processing, Processed
    request: EventRequest,  // Api | Sqs
    response: ApiGatewayProxyResponse,  // Single complete response
    response_headers: HashMap<String, String>,
    lambda_name: String,
}
```

**Store** (sam-e-invoker/src/data/store.rs):
```rust
Arc<RwLock<HashMap<lambda_name, Vec<Invocation>>>>
```

### Key Handlers

1. **POST /invoke** (`sam-e-invoker/src/invocation.rs:invoke()`)
   - Receives invocation from source
   - Stores in queue
   - Blocks waiting for `status = Processed`
   - Returns complete response

2. **GET /next** (`sam-e-invoker/src/invocation/next.rs`)
   - Lambda runtime calls to get event
   - Polls for Pending invocation
   - Changes status to Processing
   - Returns JSON event

3. **POST /:container_name/runtime/invocation/:id/response** (`sam-e-invoker/src/invocation/response.rs`)
   - Lambda sends complete response
   - Parses as ApiGatewayProxyResponse
   - Stores response
   - Changes status to Processed

---

## What Needs to Change for Streaming

### 1. **Response Storage** ✓
- **Current**: `response: ApiGatewayProxyResponse` (single object)
- **New**: Add channel for streaming events
- **File**: `sam-e-types/src/invocation.rs`
- Add: `response_streaming_channel: Option<mpsc::UnboundedSender<StreamingResponseEvent>>`

### 2. **Status State Machine** ✓
- **Current**: Pending → Processing → Processed
- **New**: Add intermediate state for streaming
- Add `Status::StreamingInProgress`
- Or use `is_streaming: bool` flag

### 3. **Lambda Response Reception** ✓
- **Current**: Receives single JSON body, parses as ApiGatewayProxyResponse
- **New**: Support AWS EventStream protocol (chunked encoding)
- **File**: `sam-e-invoker/src/invocation/response.rs`
- **Change**: 
  - Detect EventStream format (transfer-encoding: chunked)
  - Parse events sequentially
  - Send through channel instead of storing in response
  - Mark complete only on final event

### 4. **Invoke Handler Response** ✓
- **Current**: Waits for complete response, returns single response
- **New**: Return streaming response immediately
- **File**: `sam-e-invoker/src/invocation.rs`
- **Change**:
  - Create mpsc channel for streaming events
  - Pass sender to Lambda via Invocation
  - Return receiver as HTTP streaming response
  - Source adapter handles chunks as they arrive

### 5. **Source Adapter** ✓
- **Current**: Receives complete response, returns to client
- **New**: Stream response chunks back to client
- **File**: `sources/sam-e-source-apigw/src/request.rs`
- **Change**:
  - Detect streaming response from invoker
  - Parse EventStream chunks
  - Stream chunks back to original client (chunked transfer-encoding)

### 6. **EventStream Protocol Support** ✓
- **Current**: None
- **New**: Parse/encode AWS EventStream binary format
- **File**: `sam-e-invoker/src/middleware/eventstream.rs` (NEW)
- **Responsibility**: 
  - Decode incoming EventStream from Lambda
  - Encode outgoing EventStream to clients
  - Handle message framing and CRC checksums

---

## Implementation Complexity

### Low Complexity
- Adding types: StreamingResponseEvent, ResponseMode
- Adding flags: is_streaming
- Adding status: StreamingInProgress
- Using tokio channels for communication

### Medium Complexity
- Modifying /response handler to detect streaming mode
- Splitting response handling into two paths (streaming vs non-streaming)
- Adding channel creation in /invoke handler

### High Complexity
- EventStream binary protocol parsing (AWS spec)
- Message framing and checksums (CRC32)
- Backward compatibility (must support both modes)
- Testing with actual Lambda runtimes

---

## Files to Modify

**Critical Path** (for minimal streaming support):

1. **sam-e-types/src/invocation.rs**
   - Add StreamingResponseEvent struct
   - Add ResponseMode enum
   - Add channel to Invocation

2. **sam-e-invoker/src/invocation/response.rs**
   - Detect streaming mode
   - Parse EventStream
   - Send to channel

3. **sam-e-invoker/src/invocation.rs**
   - Create streaming path
   - Return channel receiver as response
   - Keep standard path for compatibility

4. **sam-e-invoker/src/middleware/eventstream.rs** (NEW)
   - EventStream parser
   - CRC verification
   - Frame extraction

5. **sources/sam-e-source-apigw/src/request.rs**
   - Detect streaming response
   - Forward chunks to client

---

## Breaking Changes

### None Required for Backward Compatibility

The framework can support both modes:
- Add `is_streaming` flag to invocation
- When false: use existing ApiGatewayProxyResponse path
- When true: use new streaming channel path
- Source adapters detect capability and choose path

All existing code continues to work unchanged.

---

## Testing Strategy

### Unit Tests
- EventStream encoder/decoder
- Channel communication
- Status transitions

### Integration Tests
- Source → Invoker → Lambda → Source (streaming mode)
- Verify chunk ordering and completeness
- Test error handling (incomplete streams)
- Test timeout scenarios

### Example Handlers
- Add streaming-enabled test handler
- Large file generator (streaming chunks)
- Progressive computation (stream results)

---

## Performance Implications

### Current (Non-Streaming)
- Source blocks until Lambda completes
- Memory usage: one full response buffered
- Latency: sum of Lambda + network times

### With Streaming
- Source returns streaming response immediately
- Memory usage: constant (streaming chunks)
- Latency: reduced (client sees first chunk sooner)
- Connection efficiency: data streams while client waits

### Trade-offs
- More complex code (EventStream protocol)
- Potential for head-of-line blocking if chunks arrive slowly
- Need proper backpressure handling on channels

---

## Risk Assessment

### Low Risk
- Type system changes (additive only)
- Source adapter changes (backward compatible)
- Flag-based mode selection

### Medium Risk
- EventStream protocol implementation (complex spec)
- Channel-based communication (deadlock potential)
- Testing with multiple Lambda runtimes

### High Risk
- Lambda runtime compatibility (may not support local streaming)
- Network/protocol edge cases (partial chunks, timeouts)
- Production load (unbounded channel sizes)

---

## Comparison: Current vs Streaming

| Aspect | Current | Streaming |
|--------|---------|-----------|
| Response Mode | All-or-nothing | Incremental chunks |
| Source Blocking | Full Lambda duration | Minimal (immediate response start) |
| Memory Peak | Full response size | Single chunk size |
| Client Latency | Lambda execution time | Network latency only |
| Complexity | Simple | Medium (EventStream protocol) |
| Backward Compat | N/A | Fully supported |

---

## Next Steps

1. **Study EventStream Protocol**
   - AWS documentation
   - Frame format, CRC computation
   - Existing Rust implementations

2. **Implement EventStream Codec**
   - Parser for incoming stream
   - Encoder for outgoing stream
   - Frame validation

3. **Prototype Streaming Path**
   - Add types and flags
   - Implement response handler split
   - Test with mock Lambda

4. **Source Adapter Integration**
   - Update API Gateway source
   - Handle streaming response format
   - Return to original client

5. **Lambda Runtime Testing**
   - Verify runtime support
   - Test with official AWS runtime
   - Custom runtime if needed

---

## Key Architectural Insights

1. **Current polling-based design** is inefficient for streaming
   - Every 100ms checks for status change
   - Works well for request-response, bad for streaming
   - Need event-driven approach (channels)

2. **Invocation as shared mutable state** 
   - Current: RwLock<HashMap<Vec<Invocation>>>
   - Streaming: channels bypass this
   - Keep store for metadata, use channels for data

3. **Source adapter abstraction**
   - All sources follow same pattern (POST /invoke)
   - Easy to extend with streaming capability
   - API Gateway source is main beneficiary

4. **Lambda runtime agnostic**
   - Framework talks Lambda runtime API
   - Runtime handles actual streaming
   - Framework must parse/format EventStream

