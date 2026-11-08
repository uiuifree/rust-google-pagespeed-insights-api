#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

mod error;
mod request;
pub mod response;
mod summary;

pub use error::GoogleApiError;
pub use request::{Category, RunPagespeedRequest, Strategy};
pub use response::{
    FieldCategory, FieldMetric, FieldMetricName, LighthouseAudit, LighthouseResult,
    LoadingExperience, PagespeedResponse, ScoreDisplayMode,
};
pub use summary::{FieldSummary, FieldValue, LabSummary};

/// Re-export of `reqwest`, for building a client to pass to
/// [`PageSpeedInsightsApi::with_client`] without adding the dependency yourself.
pub use reqwest;

use reqwest::Url;
use serde::de::DeserializeOwned;
use serde_json::Value;

const DEFAULT_BASE_URL: &str = "https://www.googleapis.com/pagespeedonline/v5";

#[derive(Debug, Clone)]
enum Credential {
    None,
    ApiKey(String),
    AccessToken(String),
}

/// Client for the PageSpeed Insights API v5.
///
/// The API works without credentials, but the shared anonymous quota is
/// small and often used up. Use an API key from a Google Cloud project with
/// the PageSpeed Insights API enabled, or an OAuth 2.0 access token.
///
/// A run takes 10 to 60 seconds. The default HTTP client has no timeout; pass
/// your own with [`with_client`](Self::with_client) if you need one.
///
/// # Example
///
/// ```no_run
/// use google_pagespeed_insights_api::{Category, PageSpeedInsightsApi, RunPagespeedRequest};
///
/// # async fn run() -> Result<(), google_pagespeed_insights_api::GoogleApiError> {
/// let api = PageSpeedInsightsApi::with_api_key("YOUR_API_KEY");
/// let request = RunPagespeedRequest::new("https://example.com/").categories(Category::ALL);
/// let response = api.run_pagespeed(&request).await?;
///
/// println!("performance: {:?}", response.category_score(Category::Performance));
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct PageSpeedInsightsApi {
    credential: Credential,
    base_url: String,
    client: reqwest::Client,
}

impl Default for PageSpeedInsightsApi {
    fn default() -> Self {
        Self::new()
    }
}

impl PageSpeedInsightsApi {
    /// Client without credentials (anonymous quota).
    pub fn new() -> Self {
        PageSpeedInsightsApi {
            credential: Credential::None,
            base_url: DEFAULT_BASE_URL.to_string(),
            client: reqwest::Client::new(),
        }
    }

    /// Client that sends an API key as the `key` query parameter.
    pub fn with_api_key<T: Into<String>>(key: T) -> Self {
        PageSpeedInsightsApi {
            credential: Credential::ApiKey(key.into()),
            ..Self::new()
        }
    }

    /// Client that sends an OAuth 2.0 access token in the `Authorization` header.
    /// The quota of the token's Google Cloud project is used.
    pub fn with_access_token<T: Into<String>>(token: T) -> Self {
        PageSpeedInsightsApi {
            credential: Credential::AccessToken(token.into()),
            ..Self::new()
        }
    }

    /// Uses the given HTTP client, for example one with a timeout or a proxy.
    pub fn with_client(mut self, client: reqwest::Client) -> Self {
        self.client = client;
        self
    }

    /// Changes the API base URL (default `https://www.googleapis.com/pagespeedonline/v5`).
    /// Mainly for tests against a mock server.
    pub fn with_base_url<T: Into<String>>(mut self, base_url: T) -> Self {
        self.base_url = base_url.into().trim_end_matches('/').to_string();
        self
    }

    /// Runs a PageSpeed analysis (`GET /runPagespeed`) and returns the typed response.
    ///
    /// # Errors
    ///
    /// * [`GoogleApiError::InvalidArgument`] when the URL does not start with `http://` or `https://`.
    /// * [`GoogleApiError::HttpStatus`] when Google returns an error, including when
    ///   Lighthouse cannot load the page (`400`) and when the quota is used up (`429`).
    /// * [`GoogleApiError::Connection`] when the request cannot be sent.
    /// * [`GoogleApiError::JsonParse`] when the response does not match the expected shape.
    pub async fn run_pagespeed(
        &self,
        request: &RunPagespeedRequest,
    ) -> Result<PagespeedResponse, GoogleApiError> {
        self.get(request).await
    }

    /// Same as [`run_pagespeed`](Self::run_pagespeed) but returns the whole JSON
    /// without converting it, for fields this crate does not type.
    pub async fn run_pagespeed_raw(
        &self,
        request: &RunPagespeedRequest,
    ) -> Result<Value, GoogleApiError> {
        self.get(request).await
    }

    fn build_url(&self, request: &RunPagespeedRequest) -> Result<Url, GoogleApiError> {
        let target = request.url();
        if !(target.starts_with("http://") || target.starts_with("https://")) {
            return Err(GoogleApiError::InvalidArgument(format!(
                "url must start with http:// or https://: {}",
                target
            )));
        }
        let mut url = Url::parse(&format!("{}/runPagespeed", self.base_url))
            .map_err(|e| GoogleApiError::InvalidArgument(format!("base url: {}", e)))?;
        {
            let mut pairs = url.query_pairs_mut();
            for (key, value) in request.query_pairs() {
                pairs.append_pair(key, &value);
            }
            if let Credential::ApiKey(key) = &self.credential {
                pairs.append_pair("key", key);
            }
        }
        Ok(url)
    }

    async fn get<T: DeserializeOwned>(
        &self,
        request: &RunPagespeedRequest,
    ) -> Result<T, GoogleApiError> {
        let url = self.build_url(request)?;
        let mut builder = self
            .client
            .get(url)
            .header(reqwest::header::ACCEPT, "application/json");
        if let Credential::AccessToken(token) = &self.credential {
            builder = builder.bearer_auth(token);
        }
        let response = builder
            .send()
            .await
            .map_err(|e| GoogleApiError::Connection(e.to_string()))?;
        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| GoogleApiError::Connection(e.to_string()))?;
        if !status.is_success() {
            return Err(GoogleApiError::HttpStatus(status.as_u16(), body));
        }
        serde_json::from_str(&body).map_err(|e| GoogleApiError::JsonParse(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_contains_parameters_in_order() {
        let api = PageSpeedInsightsApi::with_api_key("k");
        let request = RunPagespeedRequest::new("https://example.com/?a=1&b=2")
            .strategy(Strategy::Desktop)
            .categories([Category::Seo, Category::Performance, Category::Seo])
            .locale("ja");
        let url = api.build_url(&request).unwrap();
        assert_eq!(
            url.as_str(),
            "https://www.googleapis.com/pagespeedonline/v5/runPagespeed?url=https%3A%2F%2Fexample.com%2F%3Fa%3D1%26b%3D2&strategy=DESKTOP&category=SEO&category=PERFORMANCE&locale=ja&key=k"
        );
    }

    #[test]
    fn token_is_not_put_in_query() {
        let api = PageSpeedInsightsApi::with_access_token("secret");
        let url = api
            .build_url(&RunPagespeedRequest::new("https://example.com/"))
            .unwrap();
        assert!(!url.as_str().contains("secret"));
    }

    #[test]
    fn rejects_url_without_scheme() {
        let api = PageSpeedInsightsApi::new();
        let error = api
            .build_url(&RunPagespeedRequest::new("example.com"))
            .unwrap_err();
        assert!(matches!(error, GoogleApiError::InvalidArgument(_)));
    }
}
