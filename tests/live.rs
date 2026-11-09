//! Tests against the real PageSpeed Insights API.
//!
//! They are ignored by default because they need an API key and take up to a
//! minute each. Run them with:
//!
//! ```bash
//! PAGESPEED_API_KEY=... cargo test --test live -- --ignored
//! ```

use google_pagespeed_insights_api::{
    Category, GoogleApiError, PageSpeedInsightsApi, RunPagespeedRequest, Strategy, reqwest,
};
use std::time::Duration;

const TARGET: &str = "https://www.google.com/";

fn api() -> PageSpeedInsightsApi {
    let key = std::env::var("PAGESPEED_API_KEY").expect("PAGESPEED_API_KEY is not set");
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(120))
        .build()
        .unwrap();
    PageSpeedInsightsApi::with_api_key(key).with_client(client)
}

#[tokio::test]
#[ignore]
async fn all_categories_mobile() {
    let request = RunPagespeedRequest::new(TARGET)
        .strategy(Strategy::Mobile)
        .categories(Category::ALL)
        .locale("en");
    let response = api().run_pagespeed(&request).await.unwrap();

    let lighthouse = response
        .lighthouse_result
        .as_ref()
        .expect("lighthouseResult");
    assert!(
        lighthouse.runtime_error.is_none(),
        "{:?}",
        lighthouse.runtime_error
    );
    for category in Category::ALL {
        let score = response.category_score(category);
        assert!(
            score.is_some_and(|s| (0.0..=1.0).contains(&s)),
            "{category}: {score:?}"
        );
    }

    let lab = response.lab_summary().unwrap();
    assert!(lab.lcp.is_some_and(|v| v > 0.0), "{lab:?}");
    assert!(lab.cls.is_some(), "{lab:?}");

    // google.com always has field data
    let origin = response.origin_field_summary().expect("origin field data");
    assert!(origin.lcp.is_some(), "{origin:?}");
    assert!(origin.cls.is_some_and(|v| v.p75 < 10.0), "{origin:?}");
}

#[tokio::test]
#[ignore]
async fn desktop_raw() {
    let request = RunPagespeedRequest::new(TARGET).strategy(Strategy::Desktop);
    let value = api().run_pagespeed_raw(&request).await.unwrap();
    assert_eq!(
        value["lighthouseResult"]["configSettings"]["formFactor"],
        "desktop"
    );
}

#[tokio::test]
#[ignore]
async fn unreachable_page_is_http_error() {
    let request = RunPagespeedRequest::new("https://this-domain-does-not-exist.invalid/");
    let error = api().run_pagespeed(&request).await.unwrap_err();
    assert!(
        matches!(error, GoogleApiError::HttpStatus(400..=599, _)),
        "{error}"
    );
}
