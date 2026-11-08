use std::fmt::{Debug, Display, Formatter};

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
/// assert!(error.to_string().starts_with("http status 429"));
/// ```
pub enum GoogleApiError {
    /// Failed to connect, send the request, or read the response body.
    Connection(String),
    /// The response body is not the expected JSON. Holds the parser message.
    JsonParse(String),
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

impl Debug for GoogleApiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl std::error::Error for GoogleApiError {}
