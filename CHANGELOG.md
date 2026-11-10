# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-11-10

### Added

- `PageSpeedInsightsApi::run_pagespeed` and `run_pagespeed_raw` for `runPagespeed`,
  with API key, OAuth access token or anonymous access.
- Typed response: Lighthouse categories, audits and run metadata, and Chrome UX Report
  field data for the URL and its origin.
- `FieldSummary`, `LabSummary` and `LighthouseResult::failing_audits`.
- Re-export of `reqwest` for `with_client`.
- `with_quota_project` sends `x-goog-user-project`, needed with user credentials from the gcloud CLI.
- `Category::AgenticBrowsing`, `Category::Pwa` and `RunPagespeedRequest::captcha_token`.
- `GoogleApiError` with `status`, `reason`, `is_timeout` and `Error::source`. Public types are
  `#[non_exhaustive]`.
- API keys and access tokens are kept out of `Debug` output and out of connection error
  messages.
- Live tests against the real API (`tests/live.rs`, ignored by default) and a weekly CI run.
