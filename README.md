# google-pagespeed-insights-api — Rust client for the PageSpeed Insights API

[![Crates.io](https://img.shields.io/crates/v/google-pagespeed-insights-api?style=flat-square)](https://crates.io/crates/google-pagespeed-insights-api)
[![Documentation](https://img.shields.io/docsrs/google-pagespeed-insights-api?style=flat-square)](https://docs.rs/google-pagespeed-insights-api)
[![CI](https://img.shields.io/github/actions/workflow/status/uiuifree/rust-google-pagespeed-insights-api/ci.yml?branch=main&style=flat-square&label=CI)](https://github.com/uiuifree/rust-google-pagespeed-insights-api/actions/workflows/ci.yml)
[![MSRV](https://img.shields.io/badge/MSRV-1.88-blue?style=flat-square)](https://github.com/uiuifree/rust-google-pagespeed-insights-api/blob/main/Cargo.toml)
[![License](https://img.shields.io/crates/l/google-pagespeed-insights-api?style=flat-square)](https://github.com/uiuifree/rust-google-pagespeed-insights-api/blob/main/LICENSE)

Async Rust client for the
[PageSpeed Insights API v5](https://developers.google.com/speed/docs/insights/v5/get-started).
One call returns:

- **Lighthouse lab data**: category scores (performance, accessibility, best practices, SEO),
  lab metrics (FCP, LCP, TBT, CLS, Speed Index) and every audit with its details
- **Core Web Vitals field data** from the Chrome UX Report: 75th percentile of LCP, INP, CLS,
  FCP and TTFB for the URL and for its origin, over the last 28 days

The response is typed, and a few helpers give the numbers people check first:
`category_score`, `page_field_summary`, `origin_field_summary`, `lab_summary` and
`failing_audits`. Fields this crate does not type are kept as `serde_json::Value`, and
`run_pagespeed_raw` returns the whole JSON.

## Installation

```toml
[dependencies]
google-pagespeed-insights-api = "0.1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

## Quick start

```rust,no_run
use google_pagespeed_insights_api::{Category, PageSpeedInsightsApi, RunPagespeedRequest, Strategy};

#[tokio::main]
async fn main() -> Result<(), google_pagespeed_insights_api::GoogleApiError> {
    let api = PageSpeedInsightsApi::with_api_key("YOUR_API_KEY");

    let request = RunPagespeedRequest::new("https://example.com/")
        .strategy(Strategy::Mobile)
        .categories(Category::ALL)
        .locale("en");
    let response = api.run_pagespeed(&request).await?;

    // Lighthouse scores, 0.0 to 1.0
    println!("performance: {:?}", response.category_score(Category::Performance));
    println!("seo: {:?}", response.category_score(Category::Seo));

    // Core Web Vitals from real users (p75). None when the URL has no CrUX data.
    if let Some(field) = response.page_field_summary() {
        println!("LCP {:?} INP {:?} CLS {:?}", field.lcp, field.inp, field.cls);
    }

    // Lab metrics of this run, in milliseconds
    if let Some(lab) = response.lab_summary() {
        println!("lab LCP {:?} ms, TBT {:?} ms", lab.lcp, lab.tbt);
    }

    // Audits that did not pass (score below 0.9)
    if let Some(lighthouse) = &response.lighthouse_result {
        for audit in lighthouse.failing_audits(Category::Seo) {
            println!("{}: {:?}", audit.id, audit.title);
        }
    }
    Ok(())
}
```

## Authentication

| Constructor | Sent as | Quota |
| --- | --- | --- |
| `PageSpeedInsightsApi::new()` | nothing | shared anonymous quota, often exhausted (`429`) |
| `PageSpeedInsightsApi::with_api_key(key)` | `key` query parameter | your Google Cloud project |
| `PageSpeedInsightsApi::with_access_token(token)` | `Authorization: Bearer` | the token's Google Cloud project |

To get an API key, enable the PageSpeed Insights API in a Google Cloud project and create a key
under *APIs & Services → Credentials*. The default quota is 25,000 requests per day and 400 per
100 seconds.

## Field data and lab data

| | Field data (`loading_experience`) | Lab data (`lighthouse_result`) |
| --- | --- | --- |
| Source | Chrome UX Report, real Chrome users | one Lighthouse run on Google's servers |
| Period | last 28 days | now |
| Metrics | LCP, INP, CLS, FCP, TTFB (p75) | FCP, LCP, TBT, CLS, Speed Index |
| Missing when | the URL or origin has too little traffic | the page could not be loaded |

Google uses field data for the Core Web Vitals in Search. When the URL itself has too little
traffic, `loading_experience` can contain origin data with `origin_fallback = true`, or no metrics.

In the raw response the CLS percentile is the score multiplied by 100. `FieldSummary` divides
it back, so `cls.p75` is the usual value such as `0.05`.

## Errors

All methods return `Result<_, GoogleApiError>`:

- `HttpStatus(status, body)`: Google returned an error. `400` usually means Lighthouse could not
  load the page (`FAILED_DOCUMENT_REQUEST`, `NO_FCP`, ...), `403` an invalid key or a disabled API,
  `429` an exhausted quota. `body` is Google's JSON error.
- `Connection`: the request could not be sent or the body could not be read.
- `JsonParse`: the response is not the expected JSON.
- `InvalidArgument`: rejected before sending, for example a URL without `http://` or `https://`.

A run takes 10 to 60 seconds. The default client has no timeout; pass one with
`with_client(reqwest::Client::builder().timeout(...).build()?)` if you need it.

## Examples

```bash
PAGESPEED_API_KEY=... cargo run --example run -- https://example.com/ desktop
```

[`examples/run.rs`](https://github.com/uiuifree/rust-google-pagespeed-insights-api/blob/main/examples/run.rs)
prints the scores, the field and lab summaries, and the failing audits.

## Testing

```bash
cargo test
```

Tests use a mock server and a recorded-shape fixture, so they need no API key.

## License

MIT

## 日本語での概要

PageSpeed Insights API v5 の非同期 Rust クライアントです。1 回の呼び出しで、Lighthouse のスコアと監査結果（ラボデータ）と、Chrome UX Report による Core Web Vitals（LCP / INP / CLS の 75 パーセンタイル、フィールドデータ）を URL とオリジンの両方について取得できます。

API キーなしでも呼べますが、共有の枠はすぐ上限に達するため、Google Cloud で PageSpeed Insights API を有効にして API キーを発行し、`PageSpeedInsightsApi::with_api_key` を使ってください。
