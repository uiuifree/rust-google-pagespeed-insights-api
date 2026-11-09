use serde_json::Value;
use std::error::Error;
use std::fmt::{Display, Formatter};

/// Error type returned by all API calls.
///
/// The variants tell you where the call failed: before sending
/// ([`InvalidArgument`](Self::InvalidArgument)), while sending
/// ([`Connection`](Self::Connection)), in Google's answer
/// ([`HttpStatus`](Self::HttpStatus)), or while reading the answer
/// ([`JsonParse`](Self::JsonParse)).
///
/// # Example
///
/// ```
/// use google_pagespeed_insights_api::GoogleApiError;
///
/// let error = GoogleApiError::HttpStatus(429, r#"{"error":{"status":"RESOURCE_EXHAUSTED"}}"#.into());
/// assert_eq!(error.status(), Some(429));
/// assert_eq!(error.reason().as_deref(), Some("RESOURCE_EXHAUSTED"));
/// assert!(error.to_string().starts_with("http status 429"));
/// ```
#[derive(Debug)]
#[non_exhaustive]
pub enum GoogleApiError {
    /// Failed to connect, send the request, or read the response body.
    Connection(reqwest::Error),
    /// The response body is not the expected JSON.
    JsonParse(serde_json::Error),
    /// The API returned an error status. Holds the HTTP status code and the response body.
    ///
    /// Common cases: `400` when Lighthouse could not load the page
    /// (`FAILED_DOCUMENT_REQUEST`, `NO_FCP`, ...), `403` when the API key is
    /// invalid or the API is not enabled for the project, and `429` when the
    /// daily quota is used up.
    HttpStatus(u16, String),
    /// The request was rejected before sending (for example a URL without `http://` or `https://`).
    InvalidArgument(String),
}

impl GoogleApiError {
    /// Returns the HTTP status code for [`HttpStatus`](Self::HttpStatus), `None` otherwise.
    pub fn status(&self) -> Option<u16> {
        match self {
            GoogleApiError::HttpStatus(status, _) => Some(*status),
            _ => None,
        }
    }

    /// `true` when the request timed out.
    pub fn is_timeout(&self) -> bool {
        matches!(self, GoogleApiError::Connection(e) if e.is_timeout())
    }

    /// Machine-readable cause from the error body of
    /// [`HttpStatus`](Self::HttpStatus): the `reason` of the first detail that
    /// has one (`API_KEY_INVALID`), otherwise the status (`RESOURCE_EXHAUSTED`).
    pub fn reason(&self) -> Option<String> {
        let GoogleApiError::HttpStatus(_, body) = self else {
            return None;
        };
        let body: Value = serde_json::from_str(body).ok()?;
        let error = body.get("error")?;
        let reason = error
            .get("details")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .find_map(|detail| detail.get("reason"))
            .or_else(|| error.get("status"))?;
        reason.as_str().map(str::to_string)
    }
}

impl Display for GoogleApiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            GoogleApiError::Connection(e) => write!(f, "connection error: {}", e),
            GoogleApiError::JsonParse(e) => write!(f, "json parse error: {}", e),
            GoogleApiError::HttpStatus(status, body) => {
                write!(f, "http status {}: {}", status, body)
            }
            GoogleApiError::InvalidArgument(e) => write!(f, "invalid argument: {}", e),
        }
    }
}

impl Error for GoogleApiError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            GoogleApiError::Connection(e) => Some(e),
            GoogleApiError::JsonParse(e) => Some(e),
            GoogleApiError::HttpStatus(..) | GoogleApiError::InvalidArgument(_) => None,
        }
    }
}
