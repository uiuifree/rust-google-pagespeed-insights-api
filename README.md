# google-pagespeed-insights-api — Rust client for the PageSpeed Insights API

[![Crates.io](https://img.shields.io/crates/v/google-pagespeed-insights-api?style=flat-square)](https://crates.io/crates/google-pagespeed-insights-api)
[![Documentation](https://img.shields.io/docsrs/google-pagespeed-insights-api?style=flat-square)](https://docs.rs/google-pagespeed-insights-api)
[![CI](https://img.shields.io/github/actions/workflow/status/uiuifree/rust-google-pagespeed-insights-api/ci.yml?branch=main&style=flat-square&label=CI)](https://github.com/uiuifree/rust-google-pagespeed-insights-api/actions/workflows/ci.yml)
[![MSRV](https://img.shields.io/badge/MSRV-1.88-blue?style=flat-square)](https://github.com/uiuifree/rust-google-pagespeed-insights-api/blob/main/Cargo.toml)
[![License](https://img.shields.io/crates/l/google-pagespeed-insights-api?style=flat-square)](https://github.com/uiuifree/rust-google-pagespeed-insights-api/blob/main/LICENSE)

`google-pagespeed-insights-api` is an async Rust library (crate) for the
[PageSpeed Insights API v5](https://developers.google.com/speed/docs/insights/v5/get-started).
It runs Google Lighthouse on any public URL and returns, in one call:

- **Lighthouse scores** for performance, accessibility, best practices and SEO
- **Lab metrics** of the run: First Contentful Paint (FCP), Largest Contentful Paint (LCP),
  Total Blocking Time (TBT), Cumulative Layout Shift (CLS), Speed Index
- **Every Lighthouse audit** with its score, display value and details
- **Core Web Vitals field data** from the Chrome UX Report (CrUX): the 75th percentile of LCP,
  Interaction to Next Paint (INP), CLS, FCP and Time to First Byte (TTFB) from real Chrome users,
  for the URL and for its origin

Use it to monitor site speed and SEO from a scheduled job, fail a CI build when a score drops,
or give an AI agent or CLI a typed view of a page's performance.

## Table of contents

- [Features](#features)
- [What this crate does and does not do](#what-this-crate-does-and-does-not-do)
- [API coverage](#api-coverage)
- [Prerequisites](#prerequisites)
- [Installation](#installation)
- [Quick start](#quick-start)
- [Usage](#usage)
  - [Scores](#scores)
  - [Core Web Vitals (field data)](#core-web-vitals-field-data)
  - [Lab metrics](#lab-metrics)
  - [Failing audits](#failing-audits)
  - [Output as JSON](#output-as-json)
  - [Fields that are not typed](#fields-that-are-not-typed)
  - [Error handling](#error-handling)
- [Quota and rate limits](#quota-and-rate-limits)
- [Examples](#examples)
- [Testing](#testing)
- [FAQ](#faq)
- [Alternatives](#alternatives)
- [Requirements](#requirements)
- [Contributing](#contributing)
- [License](#license)
- [Links](#links)
- [日本語での概要](#日本語での概要)

## Features

- **Full coverage of the API**: `runPagespeed` with every documented parameter
  (`url`, `strategy`, `category`, `locale`, `utm_campaign`, `utm_source`)
- **Typed response** for scores, audits, run metadata and CrUX field data; unknown fields are
  ignored, so new fields from Google do not break parsing
- **Summaries** of the numbers people check first: `category_score`, `page_field_summary`,
  `origin_field_summary`, `lab_summary`, `failing_audits`
- **Serde everywhere**: every response type and summary implements `Serialize`, so results can be
  written out as JSON directly
- **API key, OAuth access token or no credentials**
- Small dependency set: `reqwest` (rustls), `serde`, `serde_json`

## What this crate does and does not do

| Does | Does not |
|------|----------|
| Build and send `runPagespeed` requests | Run Lighthouse locally |
| Parse the response into Rust types, keep the rest as `serde_json::Value` | Type the `details` of each audit (their shape depends on the audit) |
| Summarize field and lab metrics, list failing audits | Retry, queue or throttle requests |
| Return the HTTP status and body on API errors | Return CrUX history (use [`google-crux-api`](https://crates.io/crates/google-crux-api)) |

## API coverage

| PageSpeed Insights API | Method in this crate | Returns |
|------------------------|----------------------|---------|
| `GET /pagespeedonline/v5/runPagespeed` | `PageSpeedInsightsApi::run_pagespeed(&RunPagespeedRequest)` | `PagespeedResponse` |
| same | `PageSpeedInsightsApi::run_pagespeed_raw(&RunPagespeedRequest)` | `serde_json::Value` |

| Query parameter | Builder method | Values |
|-----------------|----------------|--------|
| `url` | `RunPagespeedRequest::new(url)` | `http://` or `https://` URL |
| `strategy` | `.strategy(Strategy)` | `Mobile` (default), `Desktop` |
| `category` (repeated) | `.category(Category)`, `.categories(...)` | `Performance` (default), `Accessibility`, `BestPractices`, `Seo`, or `Category::ALL` |
| `locale` | `.locale("ja")` | language of audit titles and descriptions |
| `utm_campaign`, `utm_source` | `.utm_campaign(..)`, `.utm_source(..)` | free text |
| `key` | `PageSpeedInsightsApi::with_api_key(key)` | API key |

The deprecated `captchaToken` parameter is not supported.

## Prerequisites

The API can be called without credentials, but the shared anonymous quota is often exhausted
and returns `429`. For regular use:

1. Open the [Google Cloud console](https://console.cloud.google.com/) and select or create a project.
2. Enable the **PageSpeed Insights API** under *APIs & Services → Library*.
3. Create an API key under *APIs & Services → Credentials*. Restricting it to the PageSpeed
   Insights API is recommended.

An OAuth 2.0 access token works too (`PageSpeedInsightsApi::with_access_token`). With user
credentials from the gcloud CLI, also set the project whose quota is used; without it Google
answers `403`, because the token's OAuth client belongs to Google, not to your project:

```rust,no_run
# use google_pagespeed_insights_api::PageSpeedInsightsApi;
# let token = String::new();
// token from `gcloud auth print-access-token`
let api = PageSpeedInsightsApi::with_access_token(token).with_quota_project("my-project");
```

The project needs the PageSpeed Insights API enabled. Service account tokens do not need
`with_quota_project`.

## Installation

```toml
[dependencies]
google-pagespeed-insights-api = "0.1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

Or run `cargo add google-pagespeed-insights-api`.

## Quick start

```rust,no_run
use google_pagespeed_insights_api::{Category, PageSpeedInsightsApi, RunPagespeedRequest, Strategy};

#[tokio::main]
async fn main() -> Result<(), google_pagespeed_insights_api::GoogleApiError> {
    let api = PageSpeedInsightsApi::with_api_key("YOUR_API_KEY");

    let request = RunPagespeedRequest::new("https://example.com/")
        .strategy(Strategy::Mobile)
        .categories(Category::ALL);
    let response = api.run_pagespeed(&request).await?;

    println!("performance: {:?}", response.category_score(Category::Performance));
    println!("seo: {:?}", response.category_score(Category::Seo));
    Ok(())
}
```

## Usage

The examples below assume `response` is the `PagespeedResponse` from the quick start.

### Scores

```rust
# use google_pagespeed_insights_api::{Category, PagespeedResponse};
# fn show(response: &PagespeedResponse) {
for category in Category::ALL {
    // 0.0 to 1.0; None when the category was not requested
    if let Some(score) = response.category_score(category) {
        println!("{}: {:.0}", category.lighthouse_id(), score * 100.0);
    }
}
# }
```

### Core Web Vitals (field data)

Field data comes from the Chrome UX Report and covers the last 28 days of real Chrome users.
It is what Google uses for the Core Web Vitals in Search. `FieldSummary` holds the 75th
percentile of each metric in milliseconds, with CLS as the plain score (for example `0.05`).

```rust
# use google_pagespeed_insights_api::{FieldCategory, PagespeedResponse};
# fn show(response: &PagespeedResponse) {
match response.page_field_summary() {
    Some(field) => {
        println!("overall: {:?}", field.overall); // Some(Fast | Average | Slow)
        println!("LCP p75: {:?}", field.lcp.map(|v| v.p75));
        println!("INP p75: {:?}", field.inp.map(|v| v.p75));
        println!("CLS p75: {:?}", field.cls.map(|v| v.p75));
        if field.origin_fallback {
            println!("(origin data: the URL has too little traffic)");
        }
    }
    None => println!("no field data for this URL"),
}

// Field data for the whole origin
let origin = response.origin_field_summary();
# }
```

`FieldCategory::Fast`, `Average` and `Slow` correspond to *good*, *needs improvement* and *poor*.

### Lab metrics

Lab metrics come from the single Lighthouse run. They are useful for debugging but can vary
between runs.

```rust
# use google_pagespeed_insights_api::PagespeedResponse;
# fn show(response: &PagespeedResponse) {
if let Some(lab) = response.lab_summary() {
    println!("FCP {:?} ms, LCP {:?} ms, TBT {:?} ms, CLS {:?}", lab.fcp, lab.lcp, lab.tbt, lab.cls);
}
# }
```

### Failing audits

`failing_audits` returns the scored audits of a category with a score below 0.9, which is
where Lighthouse stops showing green. Informative, manual and not-applicable audits are skipped.

```rust
# use google_pagespeed_insights_api::{Category, PagespeedResponse};
# fn show(response: &PagespeedResponse) {
if let Some(lighthouse) = &response.lighthouse_result {
    for audit in lighthouse.failing_audits(Category::Seo) {
        println!("{} ({:?}): {:?}", audit.id, audit.score, audit.title);
    }
    // Any audit by id
    let lcp = lighthouse.audit("largest-contentful-paint");
}
# }
```

### Output as JSON

All response types and summaries implement `Serialize`. This makes it easy to hand a compact
result to another program or an AI agent:

```rust
# use google_pagespeed_insights_api::{Category, PagespeedResponse};
# fn show(response: &PagespeedResponse) -> serde_json::Result<()> {
let failing: Vec<_> = response
    .lighthouse_result
    .iter()
    .flat_map(|lh| lh.failing_audits(Category::Seo))
    .map(|a| serde_json::json!({ "id": a.id, "title": a.title, "score": a.score }))
    .collect();

let summary = serde_json::json!({
    "url": response.id,
    "scores": {
        "performance": response.category_score(Category::Performance),
        "seo": response.category_score(Category::Seo),
    },
    "field": response.page_field_summary(),
    "lab": response.lab_summary(),
    "failing_seo_audits": failing,
});
println!("{}", serde_json::to_string_pretty(&summary)?);
# Ok(())
# }
```

### Fields that are not typed

Audit `details`, `i18n`, `stackPacks`, `entities` and `fullPageScreenshot` are kept as
`serde_json::Value`. `run_pagespeed_raw` returns the whole response without conversion.

### Error handling

```rust,no_run
use google_pagespeed_insights_api::{GoogleApiError, PageSpeedInsightsApi, RunPagespeedRequest};

# async fn run(api: PageSpeedInsightsApi) {
match api.run_pagespeed(&RunPagespeedRequest::new("https://example.com/")).await {
    Ok(response) => println!("{}", response.id),
    Err(GoogleApiError::HttpStatus(400, body)) => eprintln!("Lighthouse could not load the page: {body}"),
    Err(GoogleApiError::HttpStatus(429, _)) => eprintln!("quota exhausted"),
    Err(GoogleApiError::HttpStatus(status, body)) => eprintln!("{status}: {body}"),
    Err(GoogleApiError::Connection(e)) => eprintln!("could not reach Google: {e}"),
    Err(GoogleApiError::JsonParse(e)) => eprintln!("unexpected response: {e}"),
    Err(GoogleApiError::InvalidArgument(e)) => eprintln!("bad input: {e}"),
}
# }
```

| Status | Typical cause |
|--------|---------------|
| `400` | Lighthouse could not load the page (`FAILED_DOCUMENT_REQUEST`, `NO_FCP`, `ERRORED_DOCUMENT_REQUEST`, ...) |
| `403` | invalid API key, or the API is not enabled for the project |
| `429` | quota exhausted |
| `500` | Lighthouse timed out or failed on Google's side; retrying usually works |

## Quota and rate limits

With an API key the default quota is **25,000 requests per day** and **400 requests per
100 seconds** per project, free of charge. A single run takes about 10 to 60 seconds. The default
HTTP client has no timeout; set one if you need it. The crate re-exports `reqwest`:

```rust,no_run
use std::time::Duration;
use google_pagespeed_insights_api::{reqwest, PageSpeedInsightsApi};

# fn build() -> Result<(), Box<dyn std::error::Error>> {
let client = reqwest::Client::builder().timeout(Duration::from_secs(90)).build()?;
let api = PageSpeedInsightsApi::with_api_key("YOUR_API_KEY").with_client(client);
# Ok(())
# }
```

## Examples

```bash
PAGESPEED_API_KEY=... cargo run --example run -- https://example.com/ desktop
```

[`examples/run.rs`](https://github.com/uiuifree/rust-google-pagespeed-insights-api/blob/main/examples/run.rs)
prints the scores, the field and lab summaries, and the failing audits.

## Testing

```bash
cargo test                                          # mock server, no API key needed
PAGESPEED_API_KEY=... cargo test --test live -- --ignored   # real API
```

CI runs the mock tests on every push and the live tests weekly when the `PAGESPEED_API_KEY`
secret is set, so changes in Google's response format are noticed.

## FAQ

**Is an API key required?**
No, but the anonymous quota is shared and often exhausted. Use a key for anything regular.

**What is the difference between field data and lab data?**
Field data (`loading_experience`) is what real Chrome users experienced over 28 days and is what
Google uses in Search. Lab data (`lighthouse_result`) is one Lighthouse run on Google's servers
with a throttled network and CPU. Field data is missing for pages with little traffic.

**Why is the CLS value in the raw response 100 times larger?**
The API sends the CLS percentile multiplied by 100 (`12` for `0.12`). `FieldSummary` divides it
back. Lab CLS (`numericValue` of the `cumulative-layout-shift` audit) is not scaled.

**Why do scores differ between runs?**
Lab metrics depend on network and server conditions at the moment of the run. Compare several runs
or use field data for decisions.

**Can I get the CrUX history or query an origin without running Lighthouse?**
Use [`google-crux-api`](https://crates.io/crates/google-crux-api), which calls the Chrome UX
Report API directly and returns in well under a second.

**Which TLS backend is used?**
`reqwest` 0.13 with its default `rustls` backend. No OpenSSL is required.

## Alternatives

- [`google-pagespeedonline5`](https://crates.io/crates/google-pagespeedonline5): generated
  bindings from the `google-apis-rs` project, covering the same endpoint with a generic client.
- Running Lighthouse locally with Node.js gives lab data for pages that are not public, but no
  field data.

## Requirements

- Rust 1.88 or later
- A Tokio runtime

## Contributing

Issues and pull requests are welcome. See
[CONTRIBUTING.md](https://github.com/uiuifree/rust-google-pagespeed-insights-api/blob/main/CONTRIBUTING.md).

## License

MIT. See [LICENSE](https://github.com/uiuifree/rust-google-pagespeed-insights-api/blob/main/LICENSE).

## Links

- [PageSpeed Insights API documentation](https://developers.google.com/speed/docs/insights/v5/get-started)
- [runPagespeed reference](https://developers.google.com/speed/docs/insights/rest/v5/pagespeedapi/runpagespeed)
- [Core Web Vitals](https://web.dev/articles/vitals)
- [Lighthouse scoring](https://developer.chrome.com/docs/lighthouse/performance/performance-scoring)

## 日本語での概要

PageSpeed Insights API v5 の非同期 Rust クライアントです。URL を渡すと、1 回の呼び出しで次の情報を取得できます。

- Lighthouse のスコア（パフォーマンス、ユーザー補助、おすすめの方法、SEO）
- ラボデータ（FCP、LCP、TBT、CLS、Speed Index）
- すべての監査結果
- Chrome UX Report の Core Web Vitals（実ユーザーの LCP / INP / CLS などの 75 パーセンタイル）を URL とオリジンの両方について

`page_field_summary` / `lab_summary` で主要な数値をまとめて取り出せ、`failing_audits` で合格していない監査だけを一覧にできます。レスポンスはすべて `Serialize` を実装しているので、そのまま JSON にして他のプログラムや AI エージェントに渡せます。

API キーなしでも呼べますが、共有の枠はすぐ上限に達します。Google Cloud で PageSpeed Insights API を有効にして API キーを発行し、`PageSpeedInsightsApi::with_api_key` を使ってください。CrUX の推移を見たい場合や、Lighthouse を回さずに Core Web Vitals だけを取りたい場合は [`google-crux-api`](https://crates.io/crates/google-crux-api) を使ってください。
