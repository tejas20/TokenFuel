use std::time::Duration;
use chrono::{DateTime, Utc};
use reqwest::header::HeaderMap;
use reqwest::{Client, Response, StatusCode};
use tokenfuel_core::Status;

pub struct Failure {
    pub status: Status,
    pub message: String,
    pub retry_after: Option<u64>,
}

impl Failure {
    pub fn new(status: Status, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
            retry_after: None,
        }
    }
}

pub fn create_client() -> Result<Client, Failure> {
    Client::builder()
        .timeout(Duration::from_secs(20))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|_| Failure::new(Status::Offline, "Usage HTTP client unavailable."))
}

pub async fn read_bounded_bytes(mut response: Response, max_bytes: usize) -> Result<Vec<u8>, Failure> {
    if response.content_length().is_some_and(|n| n > max_bytes as u64) {
        return Err(Failure::new(Status::Unavailable, "Unexpected response size."));
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| Failure::new(Status::Offline, "Response was interrupted."))?
    {
        if bytes.len() + chunk.len() > max_bytes {
            return Err(Failure::new(Status::Unavailable, "Unexpected response size."));
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}

pub fn parse_retry_after(headers: &HeaderMap) -> Option<u64> {
    let header_str = headers.get("retry-after")?.to_str().ok()?;
    if let Ok(secs) = header_str.parse::<u64>() {
        return Some(secs);
    }
    if let Ok(dt) = DateTime::parse_from_rfc2822(header_str) {
        let diff = (dt.with_timezone(&Utc) - Utc::now()).num_seconds();
        return Some(diff.max(0) as u64);
    }
    None
}

pub fn handle_http_status(
    status: StatusCode,
    headers: &HeaderMap,
    service_name: &str,
) -> Result<(), Failure> {
    if status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN {
        return Err(Failure::new(
            Status::LoginRequired,
            format!("{service_name} rejected session credentials. Re-authenticate or update settings."),
        ));
    }
    if status == StatusCode::TOO_MANY_REQUESTS {
        let retry_after = parse_retry_after(headers);
        return Err(Failure {
            status: Status::RateLimited,
            message: format!("{service_name} asked TokenFuel to wait before polling again."),
            retry_after,
        });
    }
    if !status.is_success() {
        return Err(Failure::new(
            Status::Unavailable,
            format!("{service_name} service returned HTTP {}.", status.as_u16()),
        ));
    }
    Ok(())
}
