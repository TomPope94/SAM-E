# SAM-E Architecture Documentation Index

This directory contains comprehensive documentation of the SAM-E serverless invocation framework architecture.

## Documents

### 1. ARCHITECTURE_OVERVIEW.md (24 KB)
**Comprehensive technical architecture document**

Contents:
- Complete project structure with all directories and files
- Invoker component architecture (Axum router, endpoints)
- 7-step invocation flow with detailed explanations
- Source components overview (API Gateway, SQS, EventBridge, S3)
- Lambda Runtime API endpoints (detailed specifications)
- Current response handling (non-streaming)
- Docker compose orchestration
- CloudFormation template parsing pipeline
- End-to-end data flow example (HTTP request scenario)
- Key architectural patterns (in-memory store, polling, etc)
- Current limitations and observations
- Port mapping table

**Best for**: Understanding the complete architecture, data flow, and how components interact.

---

### 2. STREAMING_EXECUTIVE_SUMMARY.md (11 KB)
**High-level overview and business rationale**

Contents:
- Quick overview of SAM-E and current blocking model
- Why InvokeWithResponseStream matters (AWS Lambda feature)
- Current non-streaming architecture summary
- 6 specific changes needed for streaming (numbered list with file locations)
- Implementation complexity breakdown (low/medium/high)
- Critical files to modify (5 main files)
- Backward compatibility guarantees
- Testing strategy
- Performance implications comparison (table)
- Risk assessment
- Next steps (6-point roadmap)
- Key architectural insights

**Best for**: Executive briefing, understanding rationale, planning implementation scope.

---

### 3. STREAMING_IMPLEMENTATION.md (15 KB)
**Detailed technical implementation guide**

Contents:
- AWS Lambda InvokeWithResponseStream specification
- API differences table (current vs streaming)
- EventStream format specification
- 5 architectural blocking points with current code + problems + solutions
- Proposed streaming architecture with code examples:
  - New types (StreamingResponseEvent, ResponseMode)
  - EventStream middleware
  - Response handler modifications
  - Invoke handler modifications
  - Next handler modifications
- 6-phase implementation plan
- Key challenges analysis
- All files requiring changes (categorized by crate)
- Risk assessment

**Best for**: Detailed implementation planning, understanding blocking points, code structure changes.

---

### 4. ARCHITECTURE_QUICK_REFERENCE.md (6 KB)
**Fast lookup guide for key components**

Contents:
- System components table (6 components, ports, languages)
- Key invocation flow (4 phases)
- Core data structures (Invocation, Store, Config)
- Main API endpoints (5 endpoints with callers/blocking/request/response)
- Source adapter patterns (3 types: sync, async, streaming)
- Configuration (Docker environment, ports, credentials)
- Lambda Runtime API compliance (simulated vs not)
- Configuration parsing flow
- End-to-end data flow (detailed ASCII diagram)
- Polling mechanism explanation
- Key limitations (7 items)
- Link to streaming docs

**Best for**: Quick lookup, API reference, understanding flow at a glance.

---

## How to Use This Documentation

### For New Contributors
1. Start with: **ARCHITECTURE_QUICK_REFERENCE.md**
   - Get oriented on system components
   - Understand basic flow
   
2. Then read: **ARCHITECTURE_OVERVIEW.md** (sections 1-3)
   - Deep dive into invoker and sources
   - Understand current implementation
   
3. For streaming: **STREAMING_EXECUTIVE_SUMMARY.md**
   - Understand what's needed

### For Architecture Review
1. Start with: **ARCHITECTURE_OVERVIEW.md**
   - Complete picture of system
   - All component interactions
   
2. Focus on: Sections 1-5 (invoker, sources, flow, response, orchestration)

3. For performance: Section 5 & 8-9

### For Streaming Implementation
1. Start with: **STREAMING_EXECUTIVE_SUMMARY.md**
   - Business case and scope
   - Risk assessment
   
2. Reference: **STREAMING_IMPLEMENTATION.md**
   - Detailed blocking points
   - Code examples
   - Implementation plan
   
3. Use: **ARCHITECTURE_OVERVIEW.md**
   - Current code references
   - Data flow understanding

### For Source Adapter Development
1. Read: **ARCHITECTURE_QUICK_REFERENCE.md** (section "Source Adapter Patterns")
2. Reference: **ARCHITECTURE_OVERVIEW.md** (section 2)
3. Study: Existing implementations
   - sources/sam-e-source-apigw/src/request.rs
   - sources/sam-e-source-sqs/src/queues.rs

---

## Key Files Referenced

### Core Invoker Files
- `sam-e-invoker/src/main.rs` - Router setup
- `sam-e-invoker/src/invocation.rs` - /invoke handler (PRIMARY CHANGE POINT)
- `sam-e-invoker/src/invocation/next.rs` - /next handler
- `sam-e-invoker/src/invocation/response.rs` - /response handler (PRIMARY CHANGE POINT)
- `sam-e-invoker/src/data/store.rs` - In-memory queue store
- `sam-e-invoker/src/data/api.rs` - API state

### Types & Configuration
- `sam-e-types/src/invocation.rs` - Invocation type (PRIMARY CHANGE POINT)
- `sam-e-types/src/config.rs` - Top-level config
- `sam-e-types/src/config/lambda.rs` - Lambda function definition
- `sam-e-types/src/config/lambda/event.rs` - Event properties
- `sam-e-types/src/cloudformation/template.rs` - CF parsing

### Source Adapters
- `sources/sam-e-source-apigw/src/request.rs` - HTTP routing (PRIMARY CHANGE POINT)
- `sources/sam-e-source-sqs/src/queues.rs` - Queue polling
- `sources/sam-e-source-eventbridge/src/request.rs` - Event routing

---

## Technical Terms Glossary

- **Invocation**: A single execution request for a Lambda function
- **Request ID**: UUID tracking an invocation through the system
- **Store**: Arc<RwLock> in-memory HashMap storing invocations
- **Source Adapter**: Component that converts external events to Invocations
- **Blocking**: Operation waits for completion before returning
- **Polling**: Checking state every N milliseconds (100ms in this framework)
- **EventStream**: AWS binary protocol for streaming responses
- **RuntimeAPI**: Lambda's standardized interface (/next, /response, etc)

---

## Document Relationships

```
ARCHITECTURE_QUICK_REFERENCE.md
    ├─ Points to detailed explanations in ARCHITECTURE_OVERVIEW.md
    └─ Links streaming docs for advanced topics

ARCHITECTURE_OVERVIEW.md
    ├─ Comprehensive reference
    ├─ Complete codebase explanation
    └─ Section 9: Limitations (points to STREAMING_* docs)

STREAMING_EXECUTIVE_SUMMARY.md
    ├─ High-level overview
    ├─ Business rationale (Why stream?)
    ├─ Points to STREAMING_IMPLEMENTATION.md for details
    └─ Mentions specific files from ARCHITECTURE_OVERVIEW.md

STREAMING_IMPLEMENTATION.md
    ├─ Technical deep-dive
    ├─ Code examples
    ├─ References STREAMING_EXECUTIVE_SUMMARY.md for context
    └─ Points to specific files in ARCHITECTURE_OVERVIEW.md
```

---

## Key Insights Summary

### Current Architecture
- **Pattern**: Synchronous polling-based request-response
- **Mechanism**: In-memory RwLock-based queue store
- **Blocking**: Source adapters block until Lambda completes
- **Efficiency**: 100ms polling intervals, inefficient for streaming
- **Scope**: 4 event sources (API Gateway, SQS, S3, EventBridge)

### For Streaming Support
- **Core Change**: Replace polling with mpsc channels
- **Complexity**: Medium (EventStream protocol is the hard part)
- **Backward Compat**: Fully supported with flag
- **Impact**: 5 main files to modify, 1 new file to create
- **Timeline**: 6 implementation phases
- **Risk**: Low for type changes, Medium for protocol, High for production load

---

## Related External Resources

- AWS Lambda Runtime API: https://docs.aws.amazon.com/lambda/latest/dg/runtimes-api.html
- EventStream Protocol: https://smithy.rs/spec/protocol-tests/eventstreamrpc.html
- SAM Specification: https://docs.aws.amazon.com/serverless-application-model/latest/developerguide/what-is-sam.html
- Axum Web Framework: https://github.com/tokio-rs/axum

