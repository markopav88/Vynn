use std::time::{SystemTime, UNIX_EPOCH};

use crate::{Error, Result};
use crate::error::ClientError;
use backend::get_user_id_from_cookie;
use http::{Method, Uri};
use serde::Serialize;
use serde_json::{json, Value};
use serde_with::skip_serializing_none;
use uuid::Uuid;
use tower_cookies::Cookies;

pub async fn log_request(
    uuid: Uuid,
    req_method: Method,
    uri: Uri,
    service_error: Option<&Error>,
    client_error: Option<ClientError>,
    cookies: &Cookies,
) -> Result<()> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();

    let error_type = service_error.map(|e| e.as_ref().to_string());
    let error_data = serde_json::to_value(service_error)
        .ok()
        .and_then(|mut o| o.get_mut("data").map(|o| o.take()));

    // Create the request log line
    let log_line = RequestLogLine {
        uuid: uuid.to_string(),
        timestamp: timestamp.to_string(),
        req_path: uri.to_string(),
        req_method: req_method.to_string(),
        user_id: get_user_id_from_cookie(cookies),
        client_error_type: client_error.map(|ce| ce.as_ref().to_string()),
        error_type,
        error_data,
    };

    println!("  ->> log_request: \n{}", json!(log_line));

    // TODO SEND THE ABOVE LOG TO A LOGGING SERVICE
    Ok(())
}

#[skip_serializing_none] // will skip serializing of optionals that are None
#[derive(Serialize)]
struct RequestLogLine {
    // Unique identifier attributes
    uuid: String,
    timestamp: String,

    // User and context attributes
    user_id: Option<i32>,

    // Error attributes
    client_error_type: Option<String>,
    error_type: Option<String>,
    error_data: Option<Value>,

    // Http request attributes
    req_path: String,
    req_method: String,
}