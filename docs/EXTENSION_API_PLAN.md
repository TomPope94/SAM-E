# Plan: Adding Lambda Extension API Support to SAM-E

## Overview

Lambda Extensions are separate processes that run alongside your Lambda function, allowing them to integrate with monitoring, security, and governance tools. The Lambda Web Adapter we're trying to use is an **external extension** that needs to register with the Lambda runtime and participate in the invocation lifecycle.

## Current Architecture (Without Extension Support)

```
Client Request
    ↓
API Gateway Source
    ↓
Invoker (/invoke endpoint)
    ↓ (creates invocation in store)
Lambda Runtime API
    ↓ GET /next (Lambda polls for work)
Lambda Function
    ↓ POST /response (Lambda returns result)
Invoker
    ↓
Client Response
```

**Key Components:**
- **Invoker**: Manages invocation queue and state
- **Runtime API**: `/next`, `/response`, `/error` endpoints for Lambda function
- **No Extension API**: Extensions can't register or receive events

## Target Architecture (With Extension Support)

```
Container Starts
    ↓
Extension (Lambda Web Adapter)
    ├─ POST /extension/register (registers, gets extension ID)
    ├─ GET /extension/event/next (blocks waiting for events)
    └─ Starts web server on port 8080

Lambda Bootstrap
    ├─ GET /next (polls for invocations)
    └─ Processes work

Client Request
    ↓
API Gateway Source
    ↓
Invoker (/invoke endpoint)
    ↓ (creates invocation)
Extension API (event dispatch)
    ↓ INVOKE event → all registered extensions
    ↓ (extensions prepare)
Lambda Runtime API (/next)
    ↓ (Lambda gets invocation)
Lambda Web Adapter
    ↓ (forwards as HTTP to port 8080)
FastAPI/Uvicorn
    ↓ (processes request)
Lambda Web Adapter
    ↓ (returns response)
Runtime API (/response)
    ↓
Extension API (event dispatch)
    ↓ SHUTDOWN event (optional, on container stop)
Client Response
```

## Lambda Extension API Specification

### 1. Extension Registration
**Endpoint**: `POST /2020-01-01/extension/register`

**Request Headers:**
- `Lambda-Extension-Name`: Name of the extension

**Response:**
```json
{
  "functionName": "FastMCPHandlerLambda",
  "functionVersion": "$LATEST",
  "handler": "server.app"
}
```

**Response Headers:**
- `Lambda-Extension-Identifier`: UUID assigned to this extension

**Purpose:**
- Extensions call this when they start
- Invoker tracks registered extensions per Lambda
- Returns extension ID used in subsequent requests

### 2. Extension Event Polling
**Endpoint**: `GET /2020-01-01/extension/event/next`

**Request Headers:**
- `Lambda-Extension-Identifier`: UUID from registration

**Response (long-polling, blocks until event):**
```json
{
  "eventType": "INVOKE",
  "deadlineMs": 1234567890,
  "requestId": "request-id-123",
  "invokedFunctionArn": "arn:aws:lambda:...",
  "tracing": {
    "type": "X-Amzn-Trace-Id",
    "value": "..."
  }
}
```

**Event Types:**
- `INVOKE`: Lambda function is about to be invoked
- `SHUTDOWN`: Container is shutting down

**Purpose:**
- Extensions block on this endpoint waiting for lifecycle events
- Allows extensions to prepare before invocations
- Allows cleanup on shutdown

## Required Changes by Component

### 1. **sam-e-types/src/invocation.rs**

**What needs to change:**
- Add extension tracking structures

**New Types Needed:**
```rust
pub struct Extension {
    pub id: String,              // UUID
    pub name: String,            // Extension name
    pub event_sender: mpsc::UnboundedSender<ExtensionEvent>,
}

pub enum ExtensionEvent {
    Invoke {
        request_id: String,
        deadline_ms: u64,
        invoked_function_arn: String,
    },
    Shutdown {
        reason: String,
        deadline_ms: u64,
    },
}

pub struct Invocation {
    // ... existing fields
    pub extensions: Vec<Extension>,  // Extensions for this Lambda
}
```

**Why:**
- Need to track which extensions are registered per Lambda function
- Each extension needs its own event channel
- Extensions need to receive lifecycle events

### 2. **sam-e-invoker/src/extension/register.rs** (NEW FILE)

**What it needs:**
- Handle `POST /2020-01-01/extension/register` endpoint
- Generate unique extension ID (UUID)
- Look up the Lambda function name from environment
- Create event channel for this extension
- Store extension in invocation store
- Return extension ID and function metadata

**Flow:**
```rust
1. Extract Lambda-Extension-Name header
2. Determine which Lambda this extension belongs to (from request context)
3. Generate UUID for extension ID
4. Create mpsc channel for extension events
5. Create Extension struct
6. Add extension to Lambda's extension list in store
7. Return response with Lambda-Extension-Identifier header
```

**Challenges:**
- Need to identify which Lambda the extension belongs to
  - Extensions are called from within the Lambda container
  - Could use container hostname or pass Lambda name via environment
- Need thread-safe storage for extensions per Lambda

### 3. **sam-e-invoker/src/extension/event.rs** (NEW FILE)

**What it needs:**
- Handle `GET /2020-01-01/extension/event/next` endpoint
- Long-polling: block until event available
- Validate `Lambda-Extension-Identifier` header
- Wait on extension's event channel
- Return event when available (INVOKE or SHUTDOWN)

**Flow:**
```rust
1. Extract Lambda-Extension-Identifier header
2. Look up extension by ID in store
3. Block on extension's event_receiver channel
4. When event arrives, format as JSON response
5. Return event to extension
```

**Challenges:**
- Long-polling implementation (async await on channel)
- Timeout handling (extensions should re-poll if no event)
- Error handling if extension ID not found

### 4. **sam-e-invoker/src/extension/mod.rs** (NEW FILE)

**What it needs:**
- Module declaration for extension API
- Route setup for extension endpoints
- Helper functions for dispatching events to all extensions

**Functions:**
```rust
pub fn setup_extension_routes(app: Router) -> Router {
    // Add /:function_name/2020-01-01/extension/register
    // Add /:function_name/2020-01-01/extension/event/next
}

pub async fn dispatch_invoke_event(invocation: &Invocation) {
    // Send INVOKE event to all registered extensions
    // Wait for acknowledgment or timeout
}

pub async fn dispatch_shutdown_event(lambda_name: &str) {
    // Send SHUTDOWN event to all extensions for this Lambda
}
```

### 5. **sam-e-invoker/src/invocation.rs**

**What needs to change:**
- Before processing invocation, dispatch INVOKE event to extensions
- Extensions need to receive event before Lambda gets /next

**Modified Flow:**
```rust
1. Receive invocation request
2. Create invocation in store
3. **NEW: Dispatch INVOKE event to all registered extensions**
4. **NEW: Wait briefly for extensions to prepare (or timeout)**
5. Make invocation available on /next endpoint
6. Lambda processes invocation
7. Lambda returns response
8. Complete invocation
```

**Coordination:**
- Extensions should receive INVOKE event **before** Lambda polls /next
- Don't wait indefinitely for extensions (short timeout, e.g., 100ms)
- If no extensions registered, proceed immediately

### 6. **sam-e-invoker/src/main.rs**

**What needs to change:**
- Import and set up extension routes
- Add extension endpoints to the router

**Changes:**
```rust
mod extension;  // NEW

fn setup_routes() -> Router {
    // ... existing runtime API routes
    // e.g., /:function_name/next, /:function_name/response, /:function_name/error

    // NEW: Extension API routes
    .route("/:function_name/2020-01-01/extension/register",
           post(extension::register::handle))
    .route("/:function_name/2020-01-01/extension/event/next",
           get(extension::event::handle))
}
```

### 7. **Lambda Container Configuration**

**What needs to change:**
- Extension API endpoints need to be accessible at same host as Runtime API
- Currently: `AWS_LAMBDA_RUNTIME_API=http://sam-e-invoker:3030/FastMCPHandlerLambda`
- Extensions use base URL without Lambda name suffix

**Chosen Solution: Modified Option A - Function-scoped Extension API**

```
AWS_LAMBDA_RUNTIME_API=sam-e-invoker:3030

Runtime API:
  - GET  http://sam-e-invoker:3030/FastMCPHandlerLambda/next
  - POST http://sam-e-invoker:3030/FastMCPHandlerLambda/response
  - POST http://sam-e-invoker:3030/FastMCPHandlerLambda/error

Extension API:
  - POST http://sam-e-invoker:3030/FastMCPHandlerLambda/2020-01-01/extension/register
  - GET  http://sam-e-invoker:3030/FastMCPHandlerLambda/2020-01-01/extension/event/next
```

**Benefits:**
- Consistent pattern: all Lambda-specific APIs include function name in path
- Easy identification: extension registration automatically associated with correct Lambda
- No ambiguity: no need to infer Lambda name from container hostname or headers
- Simple routing: extract function name from path segment

## Event Flow Example

### Extension Registration (Container Startup)

```
1. Container starts
2. Lambda Web Adapter (extension) starts
3. POST /FastMCPHandlerLambda/2020-01-01/extension/register
   Header: Lambda-Extension-Name: lambda-web-adapter
4. Invoker:
   - Extracts function name from path: "FastMCPHandlerLambda"
   - Generates UUID: ext-12345
   - Creates event channel
   - Stores extension for this Lambda
   - Returns: Lambda-Extension-Identifier: ext-12345
5. Extension: GET /FastMCPHandlerLambda/2020-01-01/extension/event/next
   Header: Lambda-Extension-Identifier: ext-12345
   (blocks, waiting for event)
6. Lambda bootstrap starts
7. GET /FastMCPHandlerLambda/next
   (blocks, waiting for invocation)
```

### Invocation with Extension

```
1. Client sends request to API Gateway
2. API Gateway source forwards to Invoker
3. Invoker creates invocation
4. Invoker dispatches INVOKE event to extensions:
   - Looks up extensions for this Lambda
   - Sends event through each extension's channel
5. Extension receives event from /event/next:
   {
     "eventType": "INVOKE",
     "requestId": "req-123",
     ...
   }
6. Extension prepares (Lambda Web Adapter ready to forward)
7. Extension calls GET /event/next again (re-polls)
8. Lambda function receives invocation from GET /next
9. Lambda Web Adapter intercepts, forwards to port 8080
10. FastAPI processes request
11. Response flows back
```

## State Management

**Per-Lambda State:**
```rust
struct LambdaState {
    invocations: Vec<Invocation>,
    extensions: HashMap<String, Extension>,  // key: extension ID
}
```

**Global State:**
```rust
struct InvokerState {
    lambdas: HashMap<String, LambdaState>,  // key: Lambda name
}
```

## Backward Compatibility

**Important:**
- Extension API is **optional**
- Lambdas without extensions continue to work normally
- If no extensions registered, skip extension event dispatch
- Existing non-extension Lambdas unaffected

## Testing Strategy

### Phase 1: Extension Registration
- Create test extension that registers
- Verify extension ID returned
- Verify extension stored in state

### Phase 2: Event Polling
- Test long-polling behavior
- Test event dispatch to single extension
- Test timeout handling

### Phase 3: Invocation Coordination
- Test INVOKE event sent before /next
- Test Lambda Web Adapter receives events
- Test end-to-end with FastMCP handler

### Phase 4: Multi-Extension
- Test multiple extensions per Lambda
- Test all extensions receive events
- Test extension isolation (events only to correct Lambda's extensions)

## Design Decisions

1. **Extension Identification**: ✅ DECIDED
   - Extension API includes function name in path: `/:function-name/2020-01-01/extension/register`
   - Function name extracted from URL path segment
   - No need for hostname parsing or additional headers

2. **Extension Readiness**: ✅ DECIDED - Follow AWS implementation
   - Don't wait for extensions to acknowledge INVOKE events
   - Send event notification and proceed immediately
   - Extensions receive events asynchronously via long-polling

3. **Extension Cleanup**: ✅ DECIDED - Follow AWS implementation
   - Send SHUTDOWN event on container stop
   - Extensions removed when channel closes
   - No explicit unregister endpoint needed

4. **Concurrency**: ✅ DECIDED - Follow AWS implementation
   - Extensions maintain event queue via mpsc channel
   - Multiple INVOKE events can be queued if concurrent invocations occur
   - Extensions poll `/event/next` repeatedly to consume queue

## Summary of Files to Create/Modify

**New Files:**
- `sam-e-invoker/src/extension/mod.rs`
- `sam-e-invoker/src/extension/register.rs`
- `sam-e-invoker/src/extension/event.rs`

**Modified Files:**
- `sam-e-types/src/invocation.rs` - Add extension types
- `sam-e-types/Cargo.toml` - Dependencies (if needed)
- `sam-e-invoker/src/invocation.rs` - Add extension event dispatch
- `sam-e-invoker/src/main.rs` - Add extension routes

**Configuration:**
- Docker Compose may need adjustment for Runtime API URL format

## Estimated Complexity

- **Extension Registration**: Medium - Need UUID generation, state management
- **Event Polling**: Medium - Long-polling, channel management
- **Event Dispatch**: Low - Send events through channels
- **Coordination**: Medium - Ordering of events vs /next
- **Testing**: High - Need to verify extension lifecycle carefully

This is similar in scope to the streaming implementation, focusing on new endpoints and coordination rather than changing existing Lambda execution.

## Next Steps

1. Review and approve this plan
2. Decide on routing approach (Option A, B, or C)
3. Answer open questions
4. Begin implementation file-by-file
5. Test with Lambda Web Adapter + FastMCP example
