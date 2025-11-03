pub mod event;
pub mod event_host;
pub mod register;
pub mod register_host;

use anyhow::Result;
use sam_e_types::invocation::{ExtensionEvent, Invocation};
use tracing::{debug, warn};

/// Dispatch an INVOKE event to all registered extensions for an invocation
/// This is called before the Lambda function polls /next
pub async fn dispatch_invoke_event(invocation: &Invocation) -> Result<()> {
    let extensions = invocation.get_extensions();

    if extensions.is_empty() {
        debug!("No extensions registered, skipping event dispatch");
        return Ok(());
    }

    let request_id = invocation.get_request_id().to_string();
    let lambda_name = invocation.get_lambda_name();

    // Create mock ARN (in real AWS this would be the full Lambda ARN)
    let invoked_function_arn = format!(
        "arn:aws:lambda:us-east-1:123456789012:function:{}",
        lambda_name
    );

    // Calculate deadline (current time + 30 seconds default)
    let deadline_ms = (chrono::Utc::now().timestamp_millis() as u64) + 30000;

    let event = ExtensionEvent::Invoke {
        request_id: request_id.clone(),
        deadline_ms,
        invoked_function_arn,
    };

    debug!("Dispatching INVOKE event to {} extension(s)", extensions.len());

    // Send event to all extensions (non-blocking)
    invocation.send_event_to_all_extensions(event)?;

    Ok(())
}

/// Dispatch a SHUTDOWN event to all registered extensions for a Lambda
/// This is called when the container is stopping
pub async fn dispatch_shutdown_event(invocation: &Invocation, reason: String) -> Result<()> {
    let extensions = invocation.get_extensions();

    if extensions.is_empty() {
        return Ok(());
    }

    let deadline_ms = (chrono::Utc::now().timestamp_millis() as u64) + 2000; // 2s to shutdown

    let event = ExtensionEvent::Shutdown {
        reason,
        deadline_ms,
    };

    debug!("Dispatching SHUTDOWN event to {} extension(s)", extensions.len());

    if let Err(e) = invocation.send_event_to_all_extensions(event) {
        warn!("Failed to send SHUTDOWN event to extensions: {}", e);
    }

    Ok(())
}
