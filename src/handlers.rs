use axum::{
    extract::State,
    response::IntoResponse,
    http::StatusCode,
    body::Body,
};
use futures_util::StreamExt;
use std::sync::Arc;
use crate::masker;
use crate::config::AppConfig;
use reqwest::Client;
use tracing::{error, info};
use bytes::Bytes;
use tokio_stream::wrappers::ReceiverStream;

pub struct AppState {
    pub http_client: Client,
    pub config: AppConfig,
}

/// Health check endpoint for Kubernetes/Docker
pub async fn health_check() -> impl IntoResponse {
    info!("Health check requested");
    (StatusCode::OK, "OK")
}

pub async fn handle_log(
    State(state): State<Arc<AppState>>,
    body: Body,
) -> impl IntoResponse {
    // 1. Setup Streaming Pipeline via MPSC Channel
    // [ Incoming Body ] -> [ Masking Task ] -> [ tx ] ==> [ rx ] -> [ Upstream Request ]
    let (tx, rx) = tokio::sync::mpsc::channel::<Result<Bytes, std::io::Error>>(32);

    // Convert axum Body to a data stream
    let mut data_stream = body.into_data_stream();

    // 2. Spawn Background Masking Task
    tokio::spawn(async move {
        let mut buffer: Vec<u8> = Vec::with_capacity(16 * 1024);

        while let Some(chunk_result) = data_stream.next().await {
            match chunk_result {
                Ok(chunk) => {
                    buffer.extend_from_slice(&chunk);

                    // Process lines if available (Typical for logs)
                    if let Some(pos) = buffer.iter().rposition(|&b| b == b'\n') {
                        let to_process = buffer[..=pos].to_vec();
                        let tail = buffer[pos + 1..].to_vec();

                        let text = String::from_utf8_lossy(&to_process);
                        let masked = masker::apply_global_standard_masking(&text);
                        
                        if tx.send(Ok(Bytes::from(masked))).await.is_err() {
                            break;
                        }
                        buffer = tail;
                    }
                    
                    // Safety: If no newline but buffer is very large (e.g. 1MB),
                    // we must process it to prevent memory bloat, but for standard 
                    // logs and JSON objects under 2MB, we wait for completion or newline.
                }
                Err(e) => {
                    error!("Error reading body stream: {}", e);
                    break;
                }
            }
        }

        // 3. Final Flush (For single-line JSON or remaining data)
        if !buffer.is_empty() {
            let text = String::from_utf8_lossy(&buffer);
            let masked = masker::apply_global_standard_masking(&text);
            let _ = tx.send(Ok(Bytes::from(masked))).await;
        }
    });

    // 3. Forward Masked Stream to Upstream Target
    let target_url = &state.config.target.url;
    let receiver_stream = ReceiverStream::new(rx);
    let upstream_body = reqwest::Body::wrap_stream(receiver_stream);

    match state.http_client
        .post(target_url)
        .body(upstream_body)
        .send()
        .await
    {
        Ok(res) => {
            // Convert reqwest::StatusCode -> axum::http::StatusCode (different http crate versions)
            let status_code = StatusCode::from_u16(res.status().as_u16())
                .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

            // Stream response body back to caller (Transparent Proxy)
            let response_stream = res.bytes_stream();
            let response_body = Body::from_stream(response_stream);

            axum::response::Response::builder()
                .status(status_code)
                .body(response_body)
                .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
        }
        Err(e) => {
            error!("Failed to forward to {}: {}", target_url, e);
            StatusCode::BAD_GATEWAY.into_response()
        }
    }
}