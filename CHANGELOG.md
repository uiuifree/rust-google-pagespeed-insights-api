# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0]

### Added

- `PageSpeedInsightsApi::run_pagespeed` and `run_pagespeed_raw` for `runPagespeed`,
  with API key, OAuth access token or anonymous access.
- Typed response: Lighthouse categories, audits and run metadata, and Chrome UX Report
  field data for the URL and its origin.
- `FieldSummary`, `LabSummary` and `LighthouseResult::failing_audits`.
