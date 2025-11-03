# SAM-E Architecture - Quick Reference

## System Components

| Component | Port | Purpose | Language |
|-----------|------|---------|----------|
| sam-e-invoker | 3030 | Lambda Runtime API + /invoke endpoint | Rust (Axum) |
| sam-e-source-apigw | 3000 | HTTP request router | Rust (Axum) |
| sam-e-source-sqs | - | SQS message poller | Rust (tokio) |
| sam-e-source-s3 | 3001 | S3 webhook handler | Rust (Axum) |
| sam-e-source-eventbridge | 3002 | EventBridge event handler | Rust (Axum) |
| Lambda containers | varies | AWS Lambda runtime (Docker) | Multiple |

---

## Key Invocation Flow

### Request Phase
```
1. Source receives external event
2. Create Invocation { status: Pending, event_data: ... }
3. HTTP POST to http://localhost:3030/invoke
4. BLOCK waiting for response
```

### Processing Phase
```
1. Invoker receives /invoke request
2. write_invocation_to_store() -> store[lambda_name].push()
3. read_invocation_from_store() -> poll every 100ms for status = Processed
```

### Lambda Execution Phase
```
1. Lambda container GET /next
2. Invoker finds Pending invocation, sets status = Processing
3. Lambda receives event JSON, executes handler
4. Lambda POST /response with result
5. Invoker parses result, sets status = Processed
```

### Response Phase
```
1. Invoker unblocks from polling
2. Retrieve processed invocation
3. Parse response (JSON/HTML/text/binary)
4. Return HTTP response to source
5. Source returns to original client
```

---

## Core Data Structures

### Invocation (sam-e-types/src/invocation.rs)
```
Invocation {
  request_id: UUID,
  lambda_name: String,
  status: Status,  // Pending | Processing | Processed
  request: EventRequest,  // Api(ApiGatewayProxyRequest) | Sqs(SqsEvent)
  response: ApiGatewayProxyResponse,  // From Lambda
  response_headers: HashMap<String, String>,
}
```

### Store (sam-e-invoker/src/data/store.rs)
```
Store {
  queues: Arc<RwLock<HashMap<lambda_name, InvocationQueue>>>
}

InvocationQueue {
  invocations: Vec<Invocation>
}
```

### Config (sam-e-types/src/config.rs)
```
Config {
  lambdas: Vec<Lambda>,  // Functions to execute
  infrastructure: Vec<Infrastructure>,  // SQS, S3, Databases
  runtime: Runtime,  // Configuration for sources
}
```

---

## Main API Endpoints

### POST /invoke
- **Caller**: Source adapters
- **Blocking**: Yes
- **Request**: JSON Invocation
- **Response**: HTTP 200 + response body
- **Key Code**: sam-e-invoker/src/invocation.rs::invoke()

### GET /{container}/{version}/runtime/invocation/next
- **Caller**: Lambda runtime
- **Blocking**: Yes (polls every 100ms)
- **Response**: HTTP 200 + event JSON + headers
  - Header: `lambda-runtime-aws-request-id`
  - Header: `lambda-runtime-deadline-ms`
- **Status Change**: Pending → Processing
- **Key Code**: sam-e-invoker/src/invocation/next.rs::request_handler()

### POST /{container}/{version}/runtime/invocation/{id}/response
- **Caller**: Lambda runtime (success case)
- **Request**: JSON ApiGatewayProxyResponse
- **Response**: HTTP 200 OK
- **Status Change**: Processing → Processed
- **Key Code**: sam-e-invoker/src/invocation/response.rs::response_handler()

### POST /{container}/{version}/runtime/init/error
- **Caller**: Lambda runtime (on init failure)
- **Request**: JSON error details
- **Response**: HTTP 200 + error message

### POST /{container}/{version}/runtime/invocation/{id}/error
- **Caller**: Lambda runtime (on execution failure)
- **Request**: JSON error details
- **Response**: HTTP 200 + error message

---

## Source Adapter Patterns

### Synchronous Sources (API Gateway)
```
Event arrives
  ↓
Create Invocation
  ↓
POST /invoke (BLOCKS)
  ↓
Return response to client
```

### Asynchronous Sources (SQS)
```
Loop (every 500ms):
  Receive message from SQS
    ↓
  Create Invocation
    ↓
  POST /invoke (doesn't block, fire-and-forget)
    ↓
  Delete message from queue
```

### Event Streaming Sources (EventBridge)
```
Listen for PutEvents API call
  ↓
Store event in memory
  ↓
Find matching Lambda triggers
  ↓
Create Invocation
  ↓
POST /invoke
```

---

## Configuration (Docker)

### Environment Variables
- `CONFIG`: Full YAML configuration (passed as string)
- `RUST_LOG`: Log level (debug, info, warn)

### Ports Exposed
- 3030: Invoker (Lambda Runtime API + /invoke)
- 3000: API Gateway source (HTTP)
- 3001: S3 source (webhooks)
- 3002: EventBridge source (events)

### AWS Credentials
- Mounted from `$HOME/.aws/credentials:ro`
- Used by SQS/S3/EventBridge sources to access AWS APIs

---

## Lambda Runtime API Compliance

### Simulated
- GET /next - Get pending invocation
- POST /response - Send result
- POST /init/error - Init failure
- POST /invocation/error - Execution failure
- Request ID headers
- Deadline headers

### Not Simulated
- /validate-response-stream
- Response streaming (EventStream protocol)
- Layers
- Container image updates
- Telemetry API

---

## Configuration Parsing (CloudFormation → Config)

### Input
SAM template with:
- `AWS::Serverless::Function` - Lambda function
- `AWS::Serverless::Api` - API Gateway
- `AWS::SQS::Queue` - SQS queue
- Events on function (Type: Api, Type: SQS, etc)

### Processing
1. `sam-e-types::cloudformation::Template` - Parse YAML
2. `sam-e-types::cloudformation::Resource` - Extract resources
3. For each Function:
   - Extract name, image, environment variables
   - Parse Events section
   - Create EventApiProperties or EventSqsProperties
4. For each Queue/Bucket/Database:
   - Create Infrastructure entry
   - Link to function triggers

### Output
- Config struct with lambdas, infrastructure, runtime
- Passed to invoker as YAML string in CONFIG env var

---

## Data Flow: End-to-End (HTTP Request)

```
1. Client: GET /api/users
           ↓
2. sam-e-source-apigw (port 3000)
   ├─ Receive request
   ├─ Match to Lambda by path + method regex
   ├─ Create ApiGatewayProxyRequest event
   ├─ Create Invocation { request: Api(...), status: Pending }
   └─ POST http://localhost:3030/invoke
                    ↓ BLOCKS
3. sam-e-invoker (port 3030)
   ├─ write_invocation_to_store()
   ├─ read_invocation_from_store()
   │  └─ Poll every 100ms for status = Processed
   │     ↓
4. Lambda Container
   ├─ GET /next
   │  ├─ Invoker finds Pending, sets Processing
   │  └─ Returns event JSON
   │
   ├─ Execute handler code
   │
   └─ POST /response
      ├─ Send result JSON
      └─ Invoker sets status = Processed
      
   ↓ back to step 3
   
3. (continued) polling unblocks
   ├─ Get processed invocation
   ├─ Parse response (JSON/HTML/text)
   └─ Return HTTP 200 + body
                    ↑ UNBLOCKS
2. (continued) source
   └─ Forward response to API source
                    ↓
1. Client: HTTP 200 + response body
```

---

## Polling Mechanism

The framework uses 100ms polling intervals throughout:

```
Source:
  POST /invoke
    ├─ waiting (1ms)
    ├─ waiting (100ms polling interval)
    ├─ check store (1ms)
    ├─ waiting (100ms polling interval)
    ├─ check store - found! (1ms)
    └─ return response

Lambda /next:
  GET /next
    ├─ waiting (1ms)
    ├─ waiting (100ms polling interval)
    ├─ check for pending (1ms)
    ├─ waiting (100ms polling interval)
    ├─ check for pending - found! (1ms)
    └─ return event
```

**Inefficiency**: Both wait up to 100ms before checking again.

---

## Key Limitations (Pre-Streaming)

1. **Blocking source adapter** - Source hangs until Lambda completes
2. **Single complete response** - Must wait for entire response
3. **Polling-based** - Inefficient 100ms wait intervals
4. **Memory buffering** - Entire response held in memory
5. **No timeout** - Infinite wait (TODO in utils.rs)
6. **No incremental delivery** - Client waits for complete execution
7. **Rust-specific headers** - Middleware checks for "rust" user-agent

---

## For Streaming Support

See STREAMING_EXECUTIVE_SUMMARY.md and STREAMING_IMPLEMENTATION.md for detailed analysis of changes needed.

**Key Change**: 
- Replace polling with event channels (mpsc)
- Return streaming response immediately from /invoke
- Parse EventStream protocol from Lambda
- Stream chunks back to original client

