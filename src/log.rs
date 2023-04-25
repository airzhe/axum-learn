use std::time::{SystemTime, UNIX_EPOCH};

use axum::http::{Method, Uri};
use serde::Serialize;
use serde_json::{json, Value};
use serde_with::skip_serializing_none;
use uuid::Uuid;

use crate::{ctx::Ctx, error::ClientError, Error, Result};

pub async fn log_request(
    uuid: Uuid,
    req_method: Method,
    uri: Uri,
    ctx: Option<Ctx>,
    server_error: Option<&Error>,
    client_error: Option<ClientError>,
) -> Result<()> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let error_type = server_error.map(|se| se.as_ref().to_string());
    let error_data = serde_json::to_value(server_error)
        .ok()
        .and_then(|mut v| v.get_mut("data").map(|v| v.take()));

    // Create RequestLogLine
    let line = RequestLogLine {
        uuid: uuid.to_string(),
        timestamp: timestamp.to_string(),
        req_path: uri.to_string(),
        req_method: req_method.to_string(),
        client_error_type: client_error.map(|ce| ce.as_ref().to_string()),
        user_id: ctx.map(|c: Ctx| c.user_id()),
        error_type,
        error_data,
    };
    println!("  ->> log_request \n {}", json!(line));

    // Todo send to cloudwatch
    Ok(())
}

#[skip_serializing_none]
#[derive(Serialize)]
struct RequestLogLine {
    uuid: String,      // uuid string format
    timestamp: String, // (shoud be iso8601)
    // -- User and content attributes
    user_id: Option<u64>,
    // -- http request - attributes
    req_path: String,
    req_method: String,

    // -- Error - attributes
    client_error_type: Option<String>,
    error_type: Option<String>,
    error_data: Option<Value>,
}
