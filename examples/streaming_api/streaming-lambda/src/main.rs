//! Example Lambda function demonstrating streaming response capabilities
//!
//! This example shows how to use the Lambda streaming response feature to send
//! chunks of data back to the client as they become available, rather than
//! buffering the entire response.

// use axum::{
//     extract::Path,
//     http::StatusCode,
//     response::{IntoResponse, Json},
//     routing::get,
//     Router,
// };
// use serde_json::{json, Value};
// use std::time::Duration;
// use tokio::time::sleep;
// use tracing::info;
// use tracing_subscriber::EnvFilter;
//
// /// Simple non-streaming endpoint that returns a complete JSON response
// async fn normal_response() -> Json<Value> {
//     info!("normal_response() called - returning complete response");
//     Json(json!({
//         "message": "This is a normal, non-streaming response",
//         "type": "complete",
//         "timestamp": chrono::Utc::now().to_rfc3339()
//     }))
// }
//
// /// Streaming endpoint that simulates progressive data generation
// /// This would be called with the X-Amz-Invocation-Type: ResponseStream header
// async fn stream_response() -> impl IntoResponse {
//     info!("stream_response() called - preparing to stream data");
//
//     // In a real streaming implementation, you would:
//     // 1. Return chunks as they become available
//     // 2. Use the x-lambda-stream-complete header to signal completion
//     //
//     // For now, this endpoint simulates what the Lambda Runtime would receive
//     // and demonstrates the concept
//
//     let mut response_data = String::new();
//
//     // Simulate streaming chunks of data
//     for i in 1..=5 {
//         let chunk = format!(
//             "Chunk {}: Generated at {}\n",
//             i,
//             chrono::Utc::now().to_rfc3339()
//         );
//         response_data.push_str(&chunk);
//         info!("Generated chunk {}", i);
//     }
//
//     (
//         StatusCode::OK,
//         [("content-type", "text/plain")],
//         response_data,
//     )
// }
//
// /// Endpoint that demonstrates sending multiple chunks with delays
// /// This simulates a long-running process that generates data over time
// async fn chunks_response() -> impl IntoResponse {
//     info!("chunks_response() called - simulating progressive data generation");
//
//     let mut chunks = Vec::new();
//
//     // Simulate generating data over time
//     for i in 1..=3 {
//         info!("Generating chunk {} of 3", i);
//
//         let chunk = json!({
//             "chunk_number": i,
//             "timestamp": chrono::Utc::now().to_rfc3339(),
//             "data": format!("This is data chunk number {}", i),
//             "progress": format!("{}%", (i * 100) / 3)
//         });
//
//         chunks.push(chunk);
//
//         // Simulate processing time
//         if i < 3 {
//             sleep(Duration::from_millis(100)).await;
//         }
//     }
//
//     (
//         StatusCode::OK,
//         [("content-type", "application/json")],
//         Json(json!({
//             "message": "All chunks generated",
//             "chunks": chunks,
//             "total_chunks": chunks.len()
//         })),
//     )
// }
//
// /// Health check endpoint
// async fn health_check() -> (StatusCode, &'static str) {
//     info!("health_check() called");
//     (StatusCode::OK, "Streaming Lambda is healthy!")
// }
//
// /// Catch-all route for demonstration
// async fn catch_all(Path(path): Path<String>) -> Json<Value> {
//     info!("catch_all() called with path: {}", path);
//     Json(json!({
//         "message": "Path not found",
//         "path": path,
//         "available_endpoints": [
//             "/Prod/normal - Non-streaming response",
//             "/Prod/stream - Streaming response example",
//             "/Prod/chunks - Multiple chunks example",
//             "/Prod/health - Health check"
//         ]
//     }))
// }
//
// #[tokio::main]
// async fn main() {
//     // Initialize tracing
//     tracing_subscriber::fmt()
//         .with_env_filter(EnvFilter::from_default_env())
//         .without_time()
//         .init();
//
//     info!("Starting Streaming Lambda example");
//
//     info!(
//         "Lambda Runtime API env var: {:?}",
//         std::env::var("AWS_LAMBDA_RUNTIME_API")
//     );
//     info!(
//         "Lambda Runtime API env var: {:?}",
//         std::env::var("AWS_LWA_LAMBDA_RUNTIME_API_PROXY")
//     );
//
//     let app = Router::new()
//         .route("/normal", get(normal_response))
//         .route("/Prod/stream", get(stream_response))
//         .route("/Prod/chunks", get(chunks_response))
//         .route("/Prod/health", get(health_check))
//         .route("/Prod/:path", get(catch_all));
//
//     let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
//     info!("listening on {}", listener.local_addr().unwrap());
//     axum::serve(listener, app).await.unwrap();
// }

use axum::{routing::get, Router};

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(|| async { "Hello, world!" }));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
