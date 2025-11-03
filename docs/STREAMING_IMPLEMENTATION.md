# InvokeWithResponseStream API - Implementation Analysis

## AWS Lambda InvokeWithResponseStream Specification

### API Differences from InvokeAsync/Invoke

| Aspect | Current Invoke | InvokeWithResponseStream |
|--------|---|---|
| Response Model | Single response object | Stream of response events |
| Content Encoding | JSON response body | EventStream (chunked encoding) |
| Response Format | ApiGatewayProxyResponse | Lambda Response Stream Event Envelope |
| Status Code | Returned in response | Returned as event metadata |
| Streaming Headers | Not applicable | Transfer-Encoding: chunked required |
| Response Chunks | Single chunk | Multiple events with payload chunks |

### Event Stream Format

InvokeWithResponseStream uses AWS EventStream protocol with payload chunks:

```
HEADERS FRAME:
  :message-type header = "event"
  :content-type header = "application/vnd.aws.lambda.invocation-type:application/octet-stream"

PAYLOAD EVENT (repeating):
  {
    "statusCode": 200,
    "headers": { ... },
    "body": "chunk of response data"
  }

FINAL EVENT:
  {
    "statusCode": 200,
    "headers": { "content-length": "12345" }
    (no body - signals end)
  }
```

### Lambda Runtime API Enhancement

AWS Lambda Runtime API v2 supports `--response-stream` flag:

```
POST /2018-06-01/runtime/invocation/response?_X_AMZN_TRACE_ID=...
  (same endpoint, different response format when streaming enabled)

Content-Encoding: aws-chunked
Transfer-Encoding: chunked

EventStream:
  ├─ metadata event (headers, status code)
  ├─ payload event 1 (chunk 1)
  ├─ payload event 2 (chunk 2)
  └─ ... (repeating)
```

---

## Current Architecture - Blocking Points

### 1. Single Response Storage (Invocation Type)

**Current** (`sam-e-types/src/invocation.rs`):
```rust
pub struct Invocation {
    // ...
    response: ApiGatewayProxyResponse,  // Single complete response
    response_headers: HashMap<String, String>,
    // ...
}
```

**Problem**: 
- Assumes entire response ready at once
- ApiGatewayProxyResponse has `body: Option<Body>` (not streaming)
- Headers extracted before body streaming begins

**Required Change**:
- Add optional streaming response container
- New response type: StreamingResponse with event queue
- Keep ApiGatewayProxyResponse for backward compatibility

### 2. Lambda Response Reception (/response endpoint)

**Current** (`sam-e-invoker/src/invocation/response.rs`):
```rust
pub async fn response_handler(
    headers: HeaderMap,
    Path((container_name, request_id)): Path<(String, Uuid)>,
    State(api_state): State<ApiState>,
    body: Bytes,  // Single complete body
) -> impl IntoResponse {
    // ... 
    let response_data: ApiGatewayProxyResponse =
        serde_json::from_slice(&body).unwrap();  // Parses as single object
    // ...
    invocation.set_response(response_data);
}
```

**Problem**:
- Expects single JSON body parse
- No support for EventStream protocol
- Sets status to Processed only after complete response
- Cannot support chunked uploads

**Required Change**:
- Add EventStream parser
- Support chunked transfer encoding
- New endpoint or parameter to indicate streaming mode
- Queue response events as they arrive
- Set status to Processed only on final event

### 3. Source Adapter Blocking (/invoke handler)

**Current** (`sam-e-invoker/src/invocation.rs`):
```rust
pub async fn invoke(
    State(api_state): State<ApiState>,
    Json(invocation): Json<Invocation>,
) -> Result<impl IntoResponse, AppError> {
    // ...
    let processed_invocation = 
        utils::read_invocation_from_store(&store, &lambda_name, request_id).await?;
    
    let res_body = processed_invocation.get_response();  // Waits for complete response
    
    // Single response returned
    let response = if let Some(response_body) = &res_body.body {
        // ... converts single response
    }
}
```

**Problem**:
- Blocks entire source adapter waiting for complete response
- Returns single HTTP response
- No support for streaming back to client

**Required Change**:
- Support InvokeWithResponseStream mode
- Return streaming response immediately
- Pipe chunks from Lambda as they arrive
- Keep current behavior for compatibility

### 4. Store Design (Arc<RwLock>)

**Current** (`sam-e-invoker/src/data/store.rs`):
```rust
pub struct Store {
    pub queues: Arc<RwLock<InvocationQueues>>,
}
pub type InvocationQueues = HashMap<String, InvocationQueue>;
pub struct InvocationQueue {
    invocations: Vec<Invocation>,
}
```

**Problem**:
- Entire Invocation is RwLocked for updates
- Cannot stream events incrementally
- Polling-based (inefficient for streaming)

**Required Change**:
- Add event channel (mpsc) for streaming responses
- Store unbounded sender in Invocation
- Receivers held by /invoke handler
- Non-blocking event delivery

### 5. Status State Machine

**Current** (`sam-e-types/src/invocation.rs`):
```rust
pub enum Status {
    Pending,
    Processing,
    Processed,
}
```

**Problem**:
- No intermediate state for "streaming in progress"
- Cannot distinguish between partial and complete response
- Polling semantics assume discrete states

**Required Change**:
- Add new status: `StreamingInProgress` (after Processing)
- Or use separate flag for streaming mode
- Transition: Pending → Processing → (StreamingInProgress) → Processed

---

## Proposed Streaming Architecture

### New Types in sam-e-types/src/invocation.rs

```rust
// Response event for streaming
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StreamingResponseEvent {
    pub status_code: Option<u16>,
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<String>,  // Could be base64-encoded binary
    pub is_final: bool,
}

// Union type for response modes
#[derive(Debug, Clone)]
pub enum ResponseMode {
    Standard(ApiGatewayProxyResponse),
    Streaming(Vec<StreamingResponseEvent>),
}

// Updated Invocation
pub struct Invocation {
    request_id: Uuid,
    date_time: DateTime<Local>,
    status: Status,
    request: EventRequest,
    response: Option<ResponseMode>,        // Now optional
    response_streaming_channel: Option<mpsc::UnboundedSender<StreamingResponseEvent>>,
    response_headers: HashMap<String, String>,
    lambda_name: String,
    is_streaming: bool,  // Flag indicating streaming mode
}

pub enum Status {
    Pending,
    Processing,
    StreamingInProgress,  // New state
    Processed,
}
```

### New EventStream Handler in Middleware

**New file**: `sam-e-invoker/src/middleware/eventstream.rs`

```rust
pub struct EventStreamDecoder {
    // Decodes AWS EventStream protocol
    // Parses chunked transfer encoding
    // Emits StreamingResponseEvent objects
}

impl EventStreamDecoder {
    pub async fn decode_stream(body: Body) -> impl Stream<Item = Result<StreamingResponseEvent>> {
        // Parse EventStream binary format
        // Extract payload chunks
        // Return stream of events
    }
}
```

### New Response Handler for Streaming

**Modified**: `sam-e-invoker/src/invocation/response.rs`

```rust
pub async fn response_handler(
    headers: HeaderMap,
    Path((container_name, request_id)): Path<(String, Uuid)>,
    State(api_state): State<ApiState>,
    body: Bytes,
) -> impl IntoResponse {
    let store = api_state.get_store();
    
    // Check if streaming mode
    let is_streaming = detect_streaming_mode(&headers);
    
    if is_streaming {
        // NEW: Handle EventStream
        response_handler_streaming(
            container_name, 
            request_id, 
            headers, 
            body, 
            store
        ).await
    } else {
        // EXISTING: Handle standard response
        response_handler_standard(
            container_name,
            request_id,
            headers,
            body,
            store
        ).await
    }
}

async fn response_handler_streaming(
    container_name: String,
    request_id: Uuid,
    headers: HeaderMap,
    body: Body,
    store: &Store,
) -> impl IntoResponse {
    // Decode EventStream
    let event_stream = EventStreamDecoder::decode_stream(body).await;
    
    // Get invocation from store
    let mut invocation = store.get_invocation(&container_name, &request_id);
    
    // Send events through channel
    tokio::spawn(async move {
        pin_mut!(event_stream);
        
        let mut response_headers = None;
        let mut status_code = None;
        
        while let Some(event) = event_stream.next().await {
            if let Ok(streaming_event) = event {
                // Extract headers from first event
                if response_headers.is_none() {
                    response_headers = streaming_event.headers.clone();
                    status_code = streaming_event.status_code;
                }
                
                // Send event through channel
                if let Some(tx) = &invocation.response_streaming_channel {
                    let _ = tx.send(streaming_event);
                }
                
                // Check if final event
                if streaming_event.is_final {
                    break;
                }
            }
        }
        
        // Mark as complete
        invocation.set_status(Status::Processed);
    });
    
    StatusCode::OK
}
```

### Modified /invoke Handler for Streaming

**Modified**: `sam-e-invoker/src/invocation.rs`

```rust
pub async fn invoke(
    State(api_state): State<ApiState>,
    Json(invocation_req): Json<Invocation>,
) -> Result<impl IntoResponse, AppError> {
    let lambda_name = invocation_req.get_lambda_name().clone();
    let request_id = invocation_req.get_request_id().clone();
    
    let store = api_state.get_store();
    
    // Check if client requested streaming
    let supports_streaming = detect_streaming_capability(&invocation_req);
    
    if supports_streaming {
        // Create streaming response channel
        let (tx, rx) = mpsc::unbounded_channel::<StreamingResponseEvent>();
        
        // Store channel in invocation
        let mut invocation = invocation_req.clone();
        invocation.response_streaming_channel = Some(tx);
        
        utils::write_invocation_to_store(invocation, &store)?;
        
        // Return streaming response immediately
        return Ok(
            StreamingResponse {
                receiver: rx,
            }.into_response()
        );
    } else {
        // EXISTING: Standard response path
        utils::write_invocation_to_store(invocation_req, &store)?;
        
        let processed_invocation = 
            utils::read_invocation_from_store(&store, &lambda_name, request_id).await?;
        
        // ... existing code
    }
}

pub struct StreamingResponse {
    receiver: mpsc::UnboundedReceiver<StreamingResponseEvent>,
}

impl IntoResponse for StreamingResponse {
    fn into_response(self) -> Response {
        // Create streaming body from receiver
        let stream = ReceiverStream::new(self.receiver)
            .map(|event| {
                // Convert to EventStream encoded bytes
                encode_event(event)
            });
        
        (
            StatusCode::OK,
            [
                (header::TRANSFER_ENCODING, "chunked"),
                (header::CONTENT_ENCODING, "aws-chunked"),
            ],
            StreamBody::new(stream),
        ).into_response()
    }
}
```

### Modified /next Handler for Streaming Support

**Modified**: `sam-e-invoker/src/invocation/next.rs`

```rust
pub async fn request_handler(
    headers: HeaderMap,
    Path(container_name): Path<String>,
    State(api_state): State<ApiState>,
) -> impl IntoResponse {
    // UNCHANGED: Still returns event same way
    // Lambda runtime will handle streaming transparently
    
    let store = api_state.get_store();
    
    // Check for streaming capability header (optional)
    let supports_streaming = headers
        .get("X-Lambda-Response-Stream")
        .and_then(|v| v.to_str().ok())
        .map(|s| s == "true")
        .unwrap_or(false);
    
    // Get invocation (same as before)
    let invocation_to_process = find_pending_invocation(&store, &container_name).await?;
    
    // Mark streaming mode in invocation
    if supports_streaming {
        mark_streaming_mode(&store, &container_name, &invocation_to_process.request_id);
    }
    
    // Return event (same format as before)
    return_event_to_lambda(invocation_to_process);
}
```

---

## Implementation Plan

### Phase 1: Type System Changes
- Add StreamingResponseEvent type
- Add ResponseMode enum
- Update Invocation struct with streaming_channel
- Add is_streaming flag

### Phase 2: Middleware & Parsing
- Implement EventStreamDecoder
- Add event stream parsing logic
- Detect transfer-encoding: chunked

### Phase 3: Response Handler Update
- Split response_handler into standard/streaming paths
- Implement streaming path with channel forwarding
- Handle EventStream binary format

### Phase 4: Invoke Handler Streaming
- Add streaming response return type
- Create mpsc channel pair
- Implement StreamingResponse IntoResponse

### Phase 5: Source Adapter Updates
- Update API Gateway source to detect streaming
- Handle EventStream response format
- Stream back to original HTTP client

### Phase 6: Testing & Integration
- Test with AWS Lambda functions using ResponseStream
- Verify EventStream format compliance
- Integration tests with sources

---

## Key Challenges

### 1. EventStream Binary Protocol
- Not standard HTTP chunked encoding
- Requires custom frame parser
- Message envelope with metadata

### 2. Backward Compatibility
- Must support both streaming and non-streaming
- Feature flag or capability header
- Transparent fallback

### 3. Lambda Runtime API Compatibility
- Official runtime may not support local streaming yet
- May need custom runtime wrapper
- Or patch runtime environment

### 4. Source Adapter Compatibility
- API Gateway source needs EventStream support
- Must handle chunked response to clients
- Content-Type negotiation

### 5. Memory Streaming
- Current implementation could buffer unboundedly
- Need flow control on channels
- Backpressure handling

---

## Files Requiring Changes

### sam-e-types crate
- `src/invocation.rs` - Add StreamingResponseEvent, ResponseMode, channel
- `src/config/lambda/event.rs` - Add streaming capability flag (optional)

### sam-e-invoker crate
- `src/invocation.rs` - Split invoke into streaming/standard paths
- `src/invocation/response.rs` - Add streaming response handler
- `src/invocation/next.rs` - Mark streaming mode (optional)
- `src/middleware.rs` - Add EventStream decoder
- `src/middleware/eventstream.rs` (NEW) - EventStream parser

### sources/sam-e-source-apigw
- `src/request.rs` - Detect streaming request, return streamed body
- `src/main.rs` - Add streaming response handling

### Docker & Testing
- Update example handlers to support streaming
- Add streaming integration tests
- Update documentation

