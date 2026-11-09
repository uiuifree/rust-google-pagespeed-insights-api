use google_pagespeed_insights_api::{
    Category, FieldCategory, FieldMetricName, GoogleApiError, PageSpeedInsightsApi,
    PagespeedResponse, RunPagespeedRequest, ScoreDisplayMode, Strategy,
};
use httpmock::prelude::*;

const FIXTURE: &str = include_str!("fixtures/run_pagespeed_mobile.json");

fn request() -> RunPagespeedRequest {
    RunPagespeedRequest::new("https://example.com/")
        .categories([Category::Performance, Category::Seo])
        .locale("ja")
}

#[tokio::test]
async fn parses_full_response() {
    let server = MockServer::start_async().await;
    let mock = server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/runPagespeed")
                .query_param("url", "https://example.com/")
                .query_param("strategy", "MOBILE")
                .query_param("category", "PERFORMANCE")
                .query_param("category", "SEO")
                .query_param("locale", "ja")
                .query_param("key", "test-key");
            then.status(200)
                .header("content-type", "application/json")
                .body(FIXTURE);
        })
        .await;

    let api = PageSpeedInsightsApi::with_api_key("test-key").with_base_url(server.base_url());
    let response = api.run_pagespeed(&request()).await.unwrap();
    mock.assert_async().await;

    assert_eq!(response.id, "https://example.com/");
    assert_eq!(response.category_score(Category::Performance), Some(0.78));
    assert_eq!(response.category_score(Category::Seo), Some(0.9));
    assert_eq!(response.category_score(Category::Accessibility), None);

    let lighthouse = response.lighthouse_result.as_ref().unwrap();
    assert_eq!(lighthouse.lighthouse_version.as_deref(), Some("12.6.0"));
    assert_eq!(
        lighthouse.audit("hreflang").unwrap().score_display_mode,
        Some(ScoreDisplayMode::NotApplicable)
    );
    assert_eq!(
        lighthouse
            .audit("some-future-audit")
            .unwrap()
            .score_display_mode,
        Some(ScoreDisplayMode::Unknown)
    );

    let failing: Vec<&str> = lighthouse
        .failing_audits(Category::Performance)
        .iter()
        .map(|a| a.id.as_str())
        .collect();
    assert_eq!(
        failing,
        [
            "first-contentful-paint",
            "largest-contentful-paint",
            "render-blocking-resources"
        ]
    );
    let failing: Vec<&str> = lighthouse
        .failing_audits(Category::Seo)
        .iter()
        .map(|a| a.id.as_str())
        .collect();
    assert_eq!(failing, ["meta-description"]);
}

#[tokio::test]
async fn summaries() {
    let response: google_pagespeed_insights_api::PagespeedResponse =
        serde_json::from_str(FIXTURE).unwrap();

    let page = response.page_field_summary().unwrap();
    assert_eq!(page.overall, Some(FieldCategory::Average));
    assert!(!page.origin_fallback);
    assert_eq!(page.lcp.unwrap().p75, 2300.0);
    assert_eq!(page.inp.unwrap().p75, 180.0);
    assert_eq!(page.cls.unwrap().p75, 0.12);
    assert_eq!(page.cls.unwrap().category, Some(FieldCategory::Average));
    assert_eq!(page.ttfb.unwrap().p75, 820.0);

    let origin = response.origin_field_summary().unwrap();
    assert_eq!(origin.id.as_deref(), Some("https://example.com"));
    assert!(origin.inp.is_none());

    let lab = response.lab_summary().unwrap();
    assert_eq!(lab.performance_score, Some(0.78));
    assert_eq!(lab.lcp, Some(3420.1));
    assert_eq!(lab.tbt, Some(120.0));
    assert_eq!(lab.cls, Some(0.0213));
    assert_eq!(lab.server_response_time, Some(210.0));
}

#[tokio::test]
async fn sends_bearer_token() {
    let server = MockServer::start_async().await;
    let mock = server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/runPagespeed")
                .query_param("strategy", "DESKTOP")
                .header("authorization", "Bearer token-1");
            then.status(200)
                .body(r#"{"kind":"pagespeedonline#result","id":"https://example.com/"}"#);
        })
        .await;

    let api = PageSpeedInsightsApi::with_access_token("token-1").with_base_url(server.base_url());
    let response = api
        .run_pagespeed(
            &RunPagespeedRequest::new("https://example.com/").strategy(Strategy::Desktop),
        )
        .await
        .unwrap();
    mock.assert_async().await;
    assert!(response.lighthouse_result.is_none());
    assert!(response.page_field_summary().is_none());
}

#[tokio::test]
async fn http_error_keeps_status_and_body() {
    let server = MockServer::start_async().await;
    server
        .mock_async(|when, then| {
            when.method(GET).path("/runPagespeed");
            then.status(400).body(
                r#"{"error":{"code":400,"message":"Lighthouse returned error: FAILED_DOCUMENT_REQUEST.","status":"INVALID_ARGUMENT"}}"#,
            );
        })
        .await;

    let api = PageSpeedInsightsApi::new().with_base_url(server.base_url());
    let error = api.run_pagespeed(&request()).await.unwrap_err();
    assert_eq!(error.status(), Some(400));
    match error {
        GoogleApiError::HttpStatus(_, body) => assert!(body.contains("FAILED_DOCUMENT_REQUEST")),
        other => panic!("unexpected error: {other}"),
    }
}

#[tokio::test]
async fn raw_returns_untyped_json() {
    let server = MockServer::start_async().await;
    server
        .mock_async(|when, then| {
            when.method(GET).path("/runPagespeed");
            then.status(200).body(FIXTURE);
        })
        .await;

    let api = PageSpeedInsightsApi::new().with_base_url(server.base_url());
    let value = api.run_pagespeed_raw(&request()).await.unwrap();
    assert_eq!(value["version"]["minor"], 15);
}

#[tokio::test]
async fn invalid_json_is_json_parse_error() {
    let server = MockServer::start_async().await;
    server
        .mock_async(|when, then| {
            when.method(GET).path("/runPagespeed");
            then.status(200).body("<html>");
        })
        .await;

    let api = PageSpeedInsightsApi::new().with_base_url(server.base_url());
    let error = api.run_pagespeed(&request()).await.unwrap_err();
    assert!(matches!(error, GoogleApiError::JsonParse(_)));
}

#[tokio::test]
async fn sends_quota_project() {
    let server = MockServer::start_async().await;
    let mock = server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/runPagespeed")
                .header("authorization", "Bearer user-token")
                .header("x-goog-user-project", "my-project");
            then.status(200).body(r#"{"id":"https://example.com/"}"#);
        })
        .await;
    let api = PageSpeedInsightsApi::with_access_token("user-token")
        .with_quota_project("my-project")
        .with_base_url(server.base_url());
    api.run_pagespeed(&RunPagespeedRequest::new("https://example.com/"))
        .await
        .unwrap();
    mock.assert_async().await;
}

#[tokio::test]
async fn parses_every_category_and_extra_metric_fields() {
    let body = r#"{
      "id": "https://example.com/",
      "loadingExperience": {
        "metrics": {
          "LARGEST_CONTENTFUL_PAINT_MS": {"percentile": 2000, "median": 1500, "metricId": "LCP", "formFactor": "PHONE"}
        }
      },
      "lighthouseResult": {
        "categories": {
          "pwa": {"id": "pwa", "score": 0.5},
          "agentic-browsing": {"id": "agentic-browsing", "score": 3, "categoryScoreDisplayMode": "FRACTION"}
        }
      }
    }"#;
    let response: PagespeedResponse = serde_json::from_str(body).unwrap();
    assert_eq!(response.category_score(Category::Pwa), Some(0.5));
    assert_eq!(
        response.category_score(Category::AgenticBrowsing),
        Some(3.0)
    );
    let lighthouse = response.lighthouse_result.as_ref().unwrap();
    assert_eq!(
        lighthouse
            .category(Category::AgenticBrowsing)
            .unwrap()
            .category_score_display_mode
            .as_deref(),
        Some("FRACTION")
    );
    let metric = response
        .loading_experience
        .as_ref()
        .unwrap()
        .metric(FieldMetricName::LargestContentfulPaint)
        .unwrap();
    assert_eq!(metric.median, Some(1500.0));
    assert_eq!(metric.metric_id.as_deref(), Some("LCP"));
    assert_eq!(metric.form_factor.as_deref(), Some("PHONE"));
}

#[tokio::test]
async fn sends_new_category_values_and_captcha_token() {
    let server = MockServer::start_async().await;
    let mock = server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/runPagespeed")
                .query_param("category", "AGENTIC_BROWSING")
                .query_param("category", "PWA")
                .query_param("captchaToken", "tok")
                .query_param("utm_campaign", "c")
                .query_param("utm_source", "s");
            then.status(200).body(r#"{"id":"https://example.com/"}"#);
        })
        .await;
    let api = PageSpeedInsightsApi::new().with_base_url(server.base_url());
    let request = RunPagespeedRequest::new("https://example.com/")
        .categories([Category::AgenticBrowsing, Category::Pwa])
        .captcha_token("tok")
        .utm_campaign("c")
        .utm_source("s");
    api.run_pagespeed(&request).await.unwrap();
    mock.assert_async().await;
}

#[test]
fn category_names() {
    assert_eq!(Category::AgenticBrowsing.as_str(), "AGENTIC_BROWSING");
    assert_eq!(
        Category::AgenticBrowsing.lighthouse_id(),
        "agentic-browsing"
    );
    assert_eq!(Category::Pwa.as_str(), "PWA");
    assert_eq!(Category::Pwa.lighthouse_id(), "pwa");
    assert_eq!(Category::ALL.len(), 4);
}

#[tokio::test]
async fn error_details() {
    use std::error::Error;

    let server = MockServer::start_async().await;
    server
        .mock_async(|when, then| {
            when.method(GET).path("/runPagespeed");
            then.status(400).body(
                r#"{"error":{"code":400,"status":"INVALID_ARGUMENT","details":[{"reason":"API_KEY_INVALID"}]}}"#,
            );
        })
        .await;
    let api = PageSpeedInsightsApi::new().with_base_url(server.base_url());
    let error = api.run_pagespeed(&request()).await.unwrap_err();
    assert_eq!(error.reason().as_deref(), Some("API_KEY_INVALID"));
    assert!(!error.is_timeout());
    assert!(error.source().is_none());

    assert_eq!(
        GoogleApiError::HttpStatus(500, "oops".into()).reason(),
        None
    );
    let invalid = GoogleApiError::InvalidArgument("bad".into());
    assert_eq!(invalid.reason(), None);
    assert_eq!(invalid.status(), None);
    assert!(invalid.source().is_none());
    assert_eq!(invalid.to_string(), "invalid argument: bad");
}

#[tokio::test]
async fn invalid_json_has_a_source() {
    use std::error::Error;

    let server = MockServer::start_async().await;
    server
        .mock_async(|when, then| {
            when.method(GET).path("/runPagespeed");
            then.status(200).body("<html>");
        })
        .await;
    let api = PageSpeedInsightsApi::new().with_base_url(server.base_url());
    let error = api.run_pagespeed(&request()).await.unwrap_err();
    assert!(error.source().is_some());
    assert!(error.to_string().starts_with("json parse error"));
}

#[tokio::test]
async fn timeout_is_reported() {
    use std::error::Error;
    use std::time::Duration;

    let server = MockServer::start_async().await;
    server
        .mock_async(|when, then| {
            when.method(GET).path("/runPagespeed");
            then.status(200).delay(Duration::from_secs(2)).body("{}");
        })
        .await;
    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(50))
        .build()
        .unwrap();
    let api = PageSpeedInsightsApi::new()
        .with_client(client)
        .with_base_url(server.base_url());
    let error = api.run_pagespeed(&request()).await.unwrap_err();
    assert!(error.is_timeout(), "{error}");
    assert!(error.source().is_some());
    assert!(error.to_string().starts_with("connection error"));
}
