# SAM-E Serverless Invocation Framework - Architectural Overview

## Project Structure

```
SAM-E/
├── sam-e-invoker/              # Main Lambda invocation runtime (Axum-based)
│   ├── src/
│   │   ├── main.rs             # Axum router setup & Lambda Runtime API endpoints
│   │   ├── invocation.rs        # Main /invoke endpoint handler
│   │   ├── invocation/
│   │   │   ├── next.rs          # Lambda Runtime API /next endpoint
│   │   │   ├── response.rs      # Lambda Runtime API /:request_id/response endpoint
│   │   │   ├── init_error.rs    # Lambda init error handler
│   │   │   ├── invocation_error.rs  # Lambda invocation error handler
│   │   │   └── utils.rs         # Store write/read utilities
│   │   ├── data/
│   │   │   ├── store.rs         # In-memory invocation queue store (Arc<RwLock>)
│   │   │   └── api.rs           # ApiState (lambdas, infrastructure, store)
│   │   ├── middleware.rs        # CORS + header injection middleware
│   │   └── api_response.rs      # Error response handling
│   └── Cargo.toml
│
├── sam-e-types/                 # Shared types & configuration structures
│   ├── src/
│   │   ├── invocation.rs        # Invocation type (request_id, status, request, response)
│   │   ├── config.rs            # Top-level Config struct
│   │   ├── config/
│   │   │   ├── lambda.rs        # Lambda function definition
│   │   │   ├── lambda/
│   │   │   │   ├── event.rs     # Event types (API, SQS) & properties
│   │   │   │   └── docker.rs    # Docker build config
│   │   │   ├── runtime.rs       # Runtime configuration
│   │   │   └── infrastructure.rs  # Infrastructure types (SQS, S3, EventBus, etc)
│   │   └── cloudformation/
│   │       ├── template.rs      # CloudFormation template parsing
│   │       └── resource.rs      # CloudFormation resource types
│   └── Cargo.toml
│
├── sources/                     # Event source adapters
│   ├── sam-e-source-apigw/      # API Gateway source (port 3000)
│   │   └── src/
│   │       ├── main.rs
│   │       ├── request.rs       # HTTP request → Invocation converter
│   │       └── response.rs
│   ├── sam-e-source-sqs/        # SQS poller (async polling)
│   │   └── src/
│   │       ├── main.rs
│   │       └── queues.rs        # Poll loop & SQS message → Invocation converter
│   ├── sam-e-source-s3/         # S3 webhook handler
│   ├── sam-e-source-eventbridge/ # EventBridge handler (port 3002)
│   └── ... [other sources]
│
├── docker-compose.yaml          # Multi-container orchestration
├── dockerfile                   # Multi-stage build for invoker
└── examples/                    # Example templates (basic_api, basic_sqs, etc)
```

---

## 1. Invoker Component Architecture (sam-e-invoker)

### Core Axum Router Setup (main.rs)

The invoker runs on port 3030 and exposes two main route groups:

```
POST   /invoke                                    → invoke handler
       (Entry point for all invocations from sources)

GET    /{container_name}/2018-06-01/runtime/invocation/next
       → next::request_handler
       (Lambda Runtime API - blocks until pending invocation available)

POST   /{container_name}/2018-06-01/runtime/invocation/{request_id}/response
       → response::response_handler
       (Lambda Runtime API - receives function result)

POST   /{container_name}/2018-06-01/runtime/init/error
       → init_error::response_handler
       (Lambda Runtime API error endpoint)

POST   /:container_name/2018-06-01/runtime/invocation/{request_id}/error
       → invocation_error::response_handler
       (Lambda Runtime API error endpoint)

POST   /2018-06-01/runtime/invocation/{request_id}/response
       → response::response_handler (fallback route)
```

### Invocation Flow - Request to Response

#### Step 1: Invocation Entry (/invoke endpoint)
- **Handler**: `invocation::invoke()` in `src/invocation.rs`
- **Input**: JSON payload containing:
  - `request_id`: UUID
  - `lambda_name`: Target Lambda function name
  - `request`: EventRequest enum (Api | Sqs)
  - `response`: ApiGatewayProxyResponse (initially default)
  - Status: Pending

#### Step 2: Store Invocation
- **Location**: `invocation/utils.rs::write_invocation_to_store()`
- **Data Structure**: 
  ```
  Store {
    queues: Arc<RwLock<HashMap<container_name, InvocationQueue>>>
  }
  InvocationQueue {
    invocations: Vec<Invocation>
  }
  ```
- Each Lambda container has its own queue in the HashMap
- Uses RwLock for thread-safe concurrent access

#### Step 3: Wait for Lambda Processing
- **Function**: `invocation/utils.rs::read_invocation_from_store()`
- **Mechanism**: 
  - Spawns async task that polls every 100ms
  - Waits for status transition: Pending → Processing → **Processed**
  - Blocks the source adapter until Lambda completes
  - No timeout (TODO comment indicates this is missing)

#### Step 4: Lambda Runtime API - /next
- **Handler**: `invocation/next.rs::request_handler()`
- **How it works**:
  1. Lambda container (running `amazon/aws-lambda-runtime-xxx` image) calls `/next`
  2. Handler spawns task to poll store every 100ms for Pending invocation
  3. When found, status changes to **Processing**
  4. Returns HTTP 200 with JSON body and headers:
     ```
     lambda-runtime-aws-request-id: {uuid}
     lambda-runtime-deadline-ms: {timestamp}
     Body: {event_json}
     ```
  5. This blocks until invocation completes

#### Step 5: Lambda Execution
- Lambda runtime executes handler code with the event
- Function may succeed or fail

#### Step 6: Lambda Runtime API - /response
- **Handler**: `invocation/response.rs::response_handler()`
- **Input**: 
  - Request ID in path
  - Response headers
  - Response body (JSON bytes for ApiGatewayProxyResponse)
- **Processing**:
  1. Locates invocation by request_id
  2. Parses response body (expects ApiGatewayProxyResponse structure)
  3. Changes status: Processing → **Processed**
  4. Stores response and response headers
- **Return**: HTTP 200 OK

#### Step 7: Source Adapter Gets Response
- The source adapter (waiting in `invoke` handler) unblocks
- Retrieves fully processed invocation from store
- Returns response to original requester
- **Response conversion** depends on content-type header:
  - `application/json` → JSON response
  - `text/html` → HTML response  
  - `text/plain` → Text response
  - Binary → Raw bytes

---

## 2. Source Components - How They Communicate with Invoker

### Common Pattern: All sources POST to `/invoke`

Sources are external adapters running in separate Docker containers. They:
1. Listen for external events
2. Convert events to `Invocation` type
3. POST to invoker's `/invoke` endpoint
4. Wait for response (synchronously)
5. Return response to original client

### API Gateway Source (sam-e-source-apigw, port 3000)

**Architecture**:
- Listens on port 3000 for HTTP requests
- Routes all HTTP methods (GET, POST, PATCH, PUT, DELETE)
- Matches request path to registered Lambda functions

**Request Flow**:
```
HTTP Request → apigw/request.rs::handler()
  │
  ├─ Extract method, headers, path, query params, body
  │
  ├─ Find matching Lambda by path + method using regex matching
  │  (EventApiProperties has route_regex for URL pattern matching)
  │
  ├─ Create EventRequest::Api(ApiGatewayProxyRequest)
  │  with normalized path, method, headers, body, query params
  │
  ├─ Build Invocation via InvocationBuilder
  │  - request_id: new UUID
  │  - lambda_name: matched function name
  │  - request: Api(...)
  │  - status: Pending
  │
  ├─ HTTP POST to http://0.0.0.0:3030/invoke with JSON payload
  │  (blocks waiting for invoker response)
  │
  └─ Return HTTP response to original client
     (JSON, HTML, or text based on content-type header)
```

**Key Code**: `sources/sam-e-source-apigw/src/request.rs` lines 47-70

### SQS Source (sam-e-source-sqs)

**Architecture**:
- No HTTP listener (doesn't accept external requests)
- Runs continuous polling loop in background
- Checks SQS queues every 500ms for messages
- Spawned via `tokio::task::spawn()` async tasks

**Message Processing Flow**:
```
Loop every 500ms:
  For each SQS queue in config:
    │
    ├─ Call SQS ReceiveMessage API (max 10 messages)
    │
    ├─ For each message:
    │  ├─ Create SqsMessage struct
    │  ├─ Get Lambda triggers from queue config (queue.triggers.lambdas)
    │  └─ For each triggered Lambda:
    │      ├─ Create EventRequest::Sqs(SqsEvent { records: [...] })
    │      ├─ Build Invocation
    │      ├─ HTTP POST to http://0.0.0.0:3030/invoke
    │      └─ (doesn't block - fire and forget)
```

**Key Code**: `sources/sam-e-source-sqs/src/queues.rs` lines 57-150

### EventBridge Source (sam-e-source-eventbridge, port 3002)

**Architecture**:
- Listens on port 3002 for EventBridge PutEvents API calls
- Stores events in memory
- Spawns background listener for each event bus
- Dispatches events to registered Lambda functions

**Current Implementation**: Partial/incomplete - has event routing infrastructure

### S3 Source (sam-e-source-s3, port 3001)

**Architecture**:
- Webhook-based (receives S3 notifications)
- Converts S3 events to Lambda invocations

---

## 3. Lambda Invocation Flow & API Endpoints

### Request-Response Flow Diagram

```
┌─────────────────┐
│  Source Adapter │ (apigw/sqs/s3/eventbridge)
└────────┬────────┘
         │ POST /invoke
         │ { Invocation }
         │
         ▼
┌──────────────────────────┐
│   sam-e-invoker:3030     │
│                          │
│  invoke() handler        │  Stores invocation
│  ├─ write to store       │  with status: Pending
│  └─ wait for completion  │
│      (poll every 100ms)  │
└────────┬─────────────────┘
         │
         │ (Blocks here while Lambda processes)
         │
    ┌────▼─────────────────────────────┐
    │  Lambda Container                │
    │  (amazon/aws-lambda-runtime)     │
    │                                  │
    │  while True:                     │
    │    GET /next ──────────┐         │
    │                        │         │
    │  Handler receives ◄────┘         │
    │  event and executes              │
    │                                  │
    │  POST /response ──────┐          │
    │  { result }           │          │
    │                       │          │
    │  Status: Processed ◄──┘          │
    └────┬──────────────────────────────┘
         │
         ▼
    Response returned to Source
    Source returns to original client
```

### Endpoints in Detail

#### 1. `POST /invoke`
- **Caller**: Source adapters (apigw, sqs, etc)
- **Blocking**: YES - waits for Lambda completion
- **Request**: JSON Invocation object
- **Response**: HTTP 200 with response body (from Lambda)
- **Timeout**: None (infinite wait) ⚠️

#### 2. `GET /{container_name}/2018-06-01/runtime/invocation/next`
- **Caller**: Lambda container runtime
- **Blocking**: YES - waits for pending invocation
- **Response**: HTTP 200 + headers + JSON event body
  ```
  lambda-runtime-aws-request-id: {uuid}
  lambda-runtime-deadline-ms: {timestamp}
  ```
- **Behavior**: Polls every 100ms until invocation available
- **Lambda Status Transition**: Pending → Processing

#### 3. `POST /{container_name}/2018-06-01/runtime/invocation/{request_id}/response`
- **Caller**: Lambda container runtime (on success)
- **Request**: JSON response (ApiGatewayProxyResponse)
- **Response**: HTTP 200 OK
- **Lambda Status Transition**: Processing → Processed
- **Key Detail**: Extracts headers from response body (for Rust runtimes)

#### 4. Error Endpoints
- `POST /{container_name}/2018-06-01/runtime/init/error` - Init phase errors
- `POST /{container_name}/2018-06-01/runtime/invocation/{request_id}/error` - Invocation errors
- **Response**: JSON with error message

---

## 4. Current Response Handling (Non-Streaming)

### Response Structure (sam-e-types::invocation::Invocation)

```rust
pub struct Invocation {
    request_id: Uuid,
    date_time: DateTime<Local>,
    status: Status,                           // Pending, Processing, Processed
    request: EventRequest,                    // Api(ApiGatewayProxyRequest) | Sqs(SqsEvent)
    response: ApiGatewayProxyResponse,        // AWS Lambda Event structure
    response_headers: HashMap<String, String>,
    lambda_name: String,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Status {
    Pending,
    Processing,
    Processed,
}
```

### Response Handling in /invoke Handler

**Location**: `sam-e-invoker/src/invocation.rs` lines 63-94

Current logic:
1. Waits for invocation status = Processed
2. Retrieves `response: ApiGatewayProxyResponse`
3. Extracts response headers
4. Uses status code from response
5. Parses response body based on content-type:
   - Attempts JSON parsing → returns JSON response
   - Checks content-type header for "text/html" → returns HTML
   - Falls back to text response
6. Returns single HTTP response with body all at once

**Key Limitations**:
- Entire response buffered in memory before returning
- No streaming support
- Response must fit in ApiGatewayProxyResponse structure
- No incremental response chunks

### Response Body Handling

Response body comes from `ApiGatewayProxyResponse`:
```rust
pub struct ApiGatewayProxyResponse {
    pub status_code: i64,
    pub headers: HashMap<String, Value>,
    pub multi_value_headers: HashMap<String, Vec<Value>>,
    pub body: Option<Body>,    // Option<Text | Binary | Empty>
    pub is_base64_encoded: bool,
}
```

Body encoding:
- Text bodies stored as strings
- Binary bodies can be base64-encoded
- Entire response must be assembled before Lambda sends to `/response`

---

## 5. Docker Compose Orchestration

### Configuration (docker-compose.yaml)

```yaml
services:
  sam-e-invoker:
    image: tompope94/sam-e-invoker
    ports:
      - 3030:3030   # Invoker API
      - 3000:3000   # API Gateway source (if enabled)
      - 3001:3001   # S3 source (if enabled)
      - 3002:3002   # EventBridge source (if enabled)
    mem_limit: 250m
    environment:
      CONFIG: $CONFIG (YAML config passed as env var)
      RUST_LOG: debug
    volumes:
      - $HOME/.aws/credentials:/root/.aws/credentials:ro
```

### Infrastructure Setup

**Key Points**:
1. **Single invoker container** runs all components (api, sqs, s3, eventbridge handlers)
2. **Lambda containers** created separately for each function (not in compose)
3. **Config passing**: Entire config (lambdas, infrastructure, runtime) passed as YAML string in CONFIG env var
4. **AWS credentials**: Mounted as volume for local AWS SDK calls
5. **Memory limit**: 250MB for invoker

### Multi-Container Communication

```
Source Containers (3000, 3001, 3002)
            │
            │ POST /invoke
            │ (HTTP, local network)
            │
Invoker (3030) ◄─────┐
            │        │
            ├─ Manages in-memory store
            ├─ Exposes Lambda Runtime API
            │
            │ (local docker network)
            │
Lambda Containers
```

All communication over Docker network (hostname resolution works)

---

## 6. CloudFormation Template Parsing

### Template Structure (sam-e-types/src/cloudformation/)

**Input**: SAM (Serverless Application Model) YAML template
**Output**: Config object with:
- List of Lambda functions
- Infrastructure resources
- Event routing rules

### Key Parsing Files

1. **template.rs** - CloudFormation Template parsing
   - Deserializes YAML into Template struct
   - Extracts Resources section
   - Preserves unprocessed sections (parameters, outputs, etc)

2. **resource.rs** - Resource type detection
   - Parses each resource in template
   - Determines resource type (Function, Api, Queue, etc)

3. **resource/function.rs** - Lambda function parsing
   - Extracts function properties
   - Parses Events section
   - Maps event types to EventProperties

4. **resource/apigw.rs** - API Gateway parsing
   - Extracts REST API configuration
   - Base path mapping

### Parsing Flow

```
template.yaml (SAM format)
       │
       │ (loaded by sam-e-cli)
       │
       ▼
Template struct (serde_yaml deserialization)
       │
       │ For each Resource:
       ├─ If AWS::Serverless::Function
       │  └─ Extract: name, image, events, environment vars
       │     For each Event:
       │     ├─ If Type: Api
       │     │  └─ Extract: RestApiId, Path, Method
       │     │     → EventApiProperties (route_regex, base_path)
       │     └─ If Type: SQS
       │        └─ Extract: Queue
       │           → EventSqsProperties
       │
       ├─ If AWS::Serverless::Api
       │  └─ Extract: Name, StageName, Domain
       │
       ├─ If AWS::SQS::Queue
       │  └─ Extract: QueueName, attributes
       │
       └─ If AWS::S3::Bucket
          └─ Extract: BucketName, etc
       │
       ▼
Config {
  lambdas: Vec<Lambda>,
  infrastructure: Vec<Infrastructure>,
  runtime: Runtime,
  ...
}
```

### Key Structures

**Lambda**:
```rust
pub struct Lambda {
    name: String,
    image: String,
    environment_vars: HashMap<String, String>,
    events: Vec<Event>,  // API, SQS, S3, EventBridge triggers
    template_name: String,
    package_type: PackageType,
    docker_build: Option<DockerBuild>,
}

pub struct Event {
    properties: Option<EventProperties>,  // Api | Sqs
}
```

**Infrastructure** (Enum):
```rust
pub enum Infrastructure {
    Sqs(ResourceContainer<QueueInfrastructure>),
    Postgres(ResourceContainer<PostgresInfrastructure>),
    Mysql(ResourceContainer<MysqlInfrastructure>),
    S3(ResourceContainer<S3Infrastructure>),
    EventBus(ResourceContainer<EventBusInfrastructure>),
    EventRule(ResourceContainer<EventRuleInfrastructure>),
}
```

### Lambda Trigger Registration (sam-e-types/src/config.rs)

When config is loaded, `Config::set_lambdas()` links infrastructure to Lambda functions:
- For each Lambda event of type SQS
- Finds matching SQS queue in infrastructure
- Adds Lambda name to queue's triggers list
- Sources use this to know which Lambdas to invoke

**Example**:
```
Queue "MyQueue" triggers ["LambdaA", "LambdaB"]
 │
 └─ SQS source polls MyQueue
    └─ For each message:
       ├─ Invoke LambdaA
       └─ Invoke LambdaB
```

---

## 7. Data Flow Summary: End-to-End Example

### Scenario: HTTP Request to API Gateway Lambda

```
1. Client: GET /api/users
         │
         ▼
2. sam-e-source-apigw (port 3000)
   ├─ Receives request
   ├─ Matches to Lambda function "GetUsers"
   ├─ Creates ApiGatewayProxyRequest
   ├─ Wraps in Invocation { status: Pending, ... }
   │
   └─ POST http://0.0.0.0:3030/invoke
      └─ BLOCKS waiting for response
         │
         ▼
3. sam-e-invoker (port 3030)
   ├─ write_invocation_to_store()
   │  └─ store.queues[GetUsers].push(invocation)
   │
   └─ read_invocation_from_store()
      ├─ Poll every 100ms
      ├─ Wait for status.Processed
      │
      ├─ Meanwhile:
      │  ┌────────────────────────────────────────┐
      │  │ Lambda Container (GetUsers)            │
      │  │                                        │
      │  │ GET /{name}/runtime/invocation/next    │
      │  │ ├─ Find invocation.status == Pending  │
      │  │ ├─ Change to Processing               │
      │  │ └─ Return 200 + event JSON            │
      │  │                                        │
      │  │ [Handler executes]                     │
      │  │ const response = {                     │
      │  │   statusCode: 200,                     │
      │  │   body: '{"users": [...]}',            │
      │  │   headers: { ... }                     │
      │  │ }                                      │
      │  │                                        │
      │  │ POST /{name}/runtime/invocation/{id}/response
      │  │ └─ Send response JSON                  │
      │  │                                        │
      │  └────────────────────────────────────────┘
      │     Status changed to Processed
      │
      └─ Retrieve invocation
         ├─ Extract response, headers, status
         └─ Return HTTP 200 + body to API source
         
4. sam-e-source-apigw
   ├─ Receives response from invoker
   ├─ Parses as JSON/HTML/text
   └─ Returns to client: HTTP 200 + body

5. Client: Receives response data
```

---

## 8. Key Architectural Patterns

### In-Memory Store Pattern
- Single source of truth: `Arc<RwLock<HashMap<lambda_name, Vec<Invocation>>>>`
- RwLock allows concurrent reads, exclusive writes
- Polling every 100ms for state changes
- No persistence - data lost on restart

### Source-Invoker Communication
- **Synchronous blocking**: Source POSTs to /invoke and waits
- **No message queue**: Direct HTTP communication
- **Request-response pattern**: Each request waits for completion

### Lambda Runtime API Simulation
- Implements subset of AWS Lambda Runtime API
- Uses standard Lambda container images (e.g., amazon/aws-lambda-runtime-nodejs)
- Blocks on `/next` until invocation available
- Single-shot response via `/response`

### Configuration-Driven Setup
- Entire setup driven from SAM template
- Config passed as YAML string in environment
- No configuration files - all in-memory after parse

---

## 9. Limitations & Observations (Pre-Streaming)

### Current Limitations
1. **No streaming support** - entire response buffered
2. **No response streaming** - Lambda must complete execution first
3. **Single response structure** - assumes ApiGatewayProxyResponse
4. **No timeout** - invoke() waits indefinitely
5. **Memory based** - no persistence, lost on restart
6. **Single container** - all sources in one invoker
7. **No response pipelining** - strict request/response ordering
8. **Status enum rigid** - only 3 states, no intermediate states

### Hardcoded Assumptions
- Response body fits in JSON/HTML/text enum
- Lambda returns ApiGatewayProxyResponse for all event types
- Rust runtime detected by "user_agent" header check (middleware.rs:34)
- No support for multiple response formats

---

## 10. Port Mapping

| Port | Service | Purpose |
|------|---------|---------|
| 3030 | sam-e-invoker | Lambda Runtime API + /invoke |
| 3000 | sam-e-source-apigw | API Gateway events |
| 3001 | sam-e-source-s3 | S3 webhooks |
| 3002 | sam-e-source-eventbridge | EventBridge events |

---

## Key Files for Understanding Streaming Migration

1. **sam-e-invoker/src/invocation.rs** (invoke handler) - Primary response return point
2. **sam-e-invoker/src/invocation/response.rs** - Lambda response reception
3. **sam-e-invoker/src/invocation/next.rs** - Event delivery to Lambda
4. **sam-e-types/src/invocation.rs** - Invocation data structure
5. **sources/sam-e-source-apigw/src/request.rs** - Client-facing HTTP response
6. **sam-e-invoker/src/middleware.rs** - Header injection (may affect streaming)

