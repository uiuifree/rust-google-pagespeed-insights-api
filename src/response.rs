//! Response types of `pagespeedapi.runpagespeed`.
//!
//! Only the commonly used parts are typed. Large or rarely used parts
//! (`details` of an audit, `i18n`, `stackPacks`, `fullPageScreenshot`, ...) are
//! kept as [`serde_json::Value`]. Unknown fields are ignored, so new fields
//! added by Google do not break deserialization.

use crate::request::Category;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

/// Response of `pagespeedapi.runpagespeed`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct PagespeedResponse {
    /// Always `pagespeedonline#result`.
    #[serde(default)]
    pub kind: String,
    /// The final URL that was analyzed (after redirects).
    #[serde(default)]
    pub id: String,
    /// Result of the captcha check. Normally `CAPTCHA_NOT_NEEDED`.
    pub captcha_result: Option<String>,
    /// Chrome UX Report field data for the URL itself.
    ///
    /// When the URL has too little traffic, Google returns origin data here
    /// with [`LoadingExperience::origin_fallback`] set to `true`, or leaves
    /// `metrics` empty.
    pub loading_experience: Option<LoadingExperience>,
    /// Chrome UX Report field data for the whole origin.
    pub origin_loading_experience: Option<LoadingExperience>,
    /// Lighthouse lab result.
    pub lighthouse_result: Option<LighthouseResult>,
    /// Time of the analysis, RFC 3339 in UTC.
    #[serde(rename = "analysisUTCTimestamp")]
    pub analysis_utc_timestamp: Option<String>,
    /// Version of the PageSpeed Insights service.
    pub version: Option<Value>,
}

impl PagespeedResponse {
    /// Score of a Lighthouse category from `0.0` to `1.0`.
    ///
    /// Returns `None` when the category was not requested or Lighthouse could
    /// not compute it.
    pub fn category_score(&self, category: Category) -> Option<f64> {
        self.lighthouse_result.as_ref()?.category_score(category)
    }
}

/// Chrome UX Report (CrUX) field data: what real Chrome users experienced
/// over the last 28 days.
///
/// Unlike the rest of the response, this object uses snake_case keys on the wire.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LoadingExperience {
    /// The URL or origin the data belongs to.
    pub id: Option<String>,
    /// Metrics keyed by name, such as `LARGEST_CONTENTFUL_PAINT_MS`.
    /// See [`FieldMetricName`] for the known names.
    #[serde(default)]
    pub metrics: BTreeMap<String, FieldMetric>,
    /// Overall assessment of the Core Web Vitals.
    pub overall_category: Option<FieldCategory>,
    /// The URL that was requested.
    pub initial_url: Option<String>,
    /// `true` when the URL had too little data and origin data is shown instead.
    pub origin_fallback: Option<bool>,
}

impl LoadingExperience {
    /// Returns one metric by name.
    pub fn metric(&self, name: FieldMetricName) -> Option<&FieldMetric> {
        self.metrics.get(name.as_str())
    }

    /// `true` when there is at least one metric.
    pub fn has_data(&self) -> bool {
        !self.metrics.is_empty()
    }
}

/// Known metric names in [`LoadingExperience::metrics`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum FieldMetricName {
    /// `LARGEST_CONTENTFUL_PAINT_MS` (milliseconds). Core Web Vital.
    LargestContentfulPaint,
    /// `INTERACTION_TO_NEXT_PAINT` (milliseconds). Core Web Vital.
    InteractionToNextPaint,
    /// `CUMULATIVE_LAYOUT_SHIFT_SCORE`. Core Web Vital.
    /// The percentile is the score multiplied by 100 (`5` means `0.05`).
    CumulativeLayoutShift,
    /// `FIRST_CONTENTFUL_PAINT_MS` (milliseconds).
    FirstContentfulPaint,
    /// `EXPERIMENTAL_TIME_TO_FIRST_BYTE` (milliseconds).
    TimeToFirstByte,
}

impl FieldMetricName {
    /// All known metric names.
    pub const ALL: [FieldMetricName; 5] = [
        FieldMetricName::LargestContentfulPaint,
        FieldMetricName::InteractionToNextPaint,
        FieldMetricName::CumulativeLayoutShift,
        FieldMetricName::FirstContentfulPaint,
        FieldMetricName::TimeToFirstByte,
    ];

    /// The key used in the response.
    pub fn as_str(&self) -> &'static str {
        match self {
            FieldMetricName::LargestContentfulPaint => "LARGEST_CONTENTFUL_PAINT_MS",
            FieldMetricName::InteractionToNextPaint => "INTERACTION_TO_NEXT_PAINT",
            FieldMetricName::CumulativeLayoutShift => "CUMULATIVE_LAYOUT_SHIFT_SCORE",
            FieldMetricName::FirstContentfulPaint => "FIRST_CONTENTFUL_PAINT_MS",
            FieldMetricName::TimeToFirstByte => "EXPERIMENTAL_TIME_TO_FIRST_BYTE",
        }
    }
}

/// One CrUX metric.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FieldMetric {
    /// 75th percentile value. Milliseconds, except CLS which is the score × 100.
    pub percentile: Option<f64>,
    /// Share of page loads in each bucket (good / needs improvement / poor).
    #[serde(default)]
    pub distributions: Vec<FieldDistribution>,
    /// Assessment of the 75th percentile.
    pub category: Option<FieldCategory>,
    /// Median value.
    pub median: Option<f64>,
    /// Identifier of the metric.
    #[serde(rename = "metricId", alias = "metric_id")]
    pub metric_id: Option<String>,
    /// Device class of the data.
    #[serde(rename = "formFactor", alias = "form_factor")]
    pub form_factor: Option<String>,
}

/// One bucket of a [`FieldMetric`] histogram.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FieldDistribution {
    /// Lower bound (inclusive).
    pub min: Option<f64>,
    /// Upper bound (exclusive). `None` for the last bucket.
    pub max: Option<f64>,
    /// Share of page loads in this bucket, from `0.0` to `1.0`.
    pub proportion: Option<f64>,
}

/// Assessment of field data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum FieldCategory {
    /// Good.
    #[serde(rename = "FAST")]
    Fast,
    /// Needs improvement.
    #[serde(rename = "AVERAGE")]
    Average,
    /// Poor.
    #[serde(rename = "SLOW")]
    Slow,
    /// No assessment.
    #[serde(rename = "NONE")]
    NoData,
    /// A value this crate does not know yet.
    #[serde(other)]
    Unknown,
}

/// Lighthouse lab result.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LighthouseResult {
    /// The URL that was requested.
    pub requested_url: Option<String>,
    /// URL of the main document after redirects.
    pub main_document_url: Option<String>,
    /// URL shown in the address bar at the end of the run.
    pub final_displayed_url: Option<String>,
    /// Deprecated by Google in favor of `mainDocumentUrl`.
    pub final_url: Option<String>,
    /// Lighthouse version, such as `12.6.0`.
    pub lighthouse_version: Option<String>,
    /// User agent of the emulated browser.
    pub user_agent: Option<String>,
    /// Time of the run, RFC 3339.
    pub fetch_time: Option<String>,
    /// Settings Lighthouse ran with.
    pub config_settings: Option<ConfigSettings>,
    /// Warnings about the run (for example a redirect or a slow server).
    #[serde(default)]
    pub run_warnings: Vec<Value>,
    /// Set when Lighthouse failed to analyze the page.
    pub runtime_error: Option<RuntimeError>,
    /// Audits keyed by id, such as `largest-contentful-paint` or `document-title`.
    #[serde(default)]
    pub audits: BTreeMap<String, LighthouseAudit>,
    /// Categories that were requested.
    #[serde(default)]
    pub categories: Categories,
    /// Groups referenced by [`AuditRef::group`].
    #[serde(default)]
    pub category_groups: BTreeMap<String, CategoryGroup>,
    /// Total run time.
    pub timing: Option<Timing>,
    /// Information about the test machine.
    pub environment: Option<Value>,
    /// Localized strings.
    pub i18n: Option<Value>,
    /// Framework specific advice.
    pub stack_packs: Option<Value>,
    /// Screenshot of the whole page (base64 image and node rectangles).
    pub full_page_screenshot: Option<Value>,
    /// Third-party entities found on the page.
    pub entities: Option<Value>,
}

impl LighthouseResult {
    /// The category object for `category`, if it was requested.
    pub fn category(&self, category: Category) -> Option<&LighthouseCategory> {
        match category {
            Category::Performance => self.categories.performance.as_ref(),
            Category::Accessibility => self.categories.accessibility.as_ref(),
            Category::BestPractices => self.categories.best_practices.as_ref(),
            Category::Seo => self.categories.seo.as_ref(),
            Category::Pwa => self.categories.pwa.as_ref(),
            Category::AgenticBrowsing => self.categories.agentic_browsing.as_ref(),
        }
    }

    /// Score of a category from `0.0` to `1.0`.
    pub fn category_score(&self, category: Category) -> Option<f64> {
        self.category(category)?.score
    }

    /// Returns one audit by id.
    pub fn audit(&self, id: &str) -> Option<&LighthouseAudit> {
        self.audits.get(id)
    }

    /// Audits of `category` that did not pass, in the order the category lists them.
    ///
    /// An audit counts as failing when it has a score below `0.9` (the
    /// threshold Lighthouse uses for a green result) and its
    /// [`ScoreDisplayMode`] is binary, numeric or metric savings.
    /// Informative, manual and not-applicable audits are never returned.
    pub fn failing_audits(&self, category: Category) -> Vec<&LighthouseAudit> {
        let Some(category) = self.category(category) else {
            return Vec::new();
        };
        category
            .audit_refs
            .iter()
            .filter_map(|r| self.audits.get(&r.id))
            .filter(|audit| audit.is_failing())
            .collect()
    }
}

/// Settings Lighthouse ran with.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ConfigSettings {
    /// `mobile` or `desktop`.
    pub form_factor: Option<String>,
    /// Locale of the strings in the result.
    pub locale: Option<String>,
    /// Categories that were run.
    pub only_categories: Option<Vec<String>>,
    /// Channel that started the run (`lr` for PageSpeed Insights).
    pub channel: Option<String>,
}

/// Why Lighthouse could not analyze the page.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct RuntimeError {
    /// Error code, such as `NO_FCP` or `ERRORED_DOCUMENT_REQUEST`.
    pub code: Option<String>,
    /// Human readable message.
    pub message: Option<String>,
}

/// Timing of the run.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Timing {
    /// Total time in milliseconds.
    pub total: Option<f64>,
}

/// The categories of a run. Only requested categories are present.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Categories {
    /// Performance.
    pub performance: Option<LighthouseCategory>,
    /// Accessibility.
    pub accessibility: Option<LighthouseCategory>,
    /// Best practices.
    #[serde(rename = "best-practices")]
    pub best_practices: Option<LighthouseCategory>,
    /// SEO.
    pub seo: Option<LighthouseCategory>,
    /// Progressive Web App.
    pub pwa: Option<LighthouseCategory>,
    /// Agentic browsing.
    #[serde(rename = "agentic-browsing")]
    pub agentic_browsing: Option<LighthouseCategory>,
}

/// One Lighthouse category.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LighthouseCategory {
    /// Category id, such as `performance` or `best-practices`.
    #[serde(default)]
    pub id: String,
    /// Localized title.
    pub title: Option<String>,
    /// Localized description.
    pub description: Option<String>,
    /// Score from `0.0` to `1.0`. `None` when it could not be computed.
    pub score: Option<f64>,
    /// How the score is meant to be displayed: `GAUGE` or `FRACTION`.
    pub category_score_display_mode: Option<String>,
    /// Description of the audits that must be checked by hand.
    pub manual_description: Option<String>,
    /// Audits that make up the category.
    #[serde(default)]
    pub audit_refs: Vec<AuditRef>,
}

/// Reference from a category to an audit.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct AuditRef {
    /// Audit id, a key of [`LighthouseResult::audits`].
    #[serde(default)]
    pub id: String,
    /// Weight of the audit in the category score.
    pub weight: Option<f64>,
    /// Group id, a key of [`LighthouseResult::category_groups`].
    pub group: Option<String>,
    /// Short name of a metric, such as `LCP` or `TBT`.
    pub acronym: Option<String>,
    /// Metrics this audit helps to improve.
    pub relevant_audits: Option<Vec<String>>,
}

/// A group of audits inside a category.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct CategoryGroup {
    /// Localized title.
    pub title: Option<String>,
    /// Localized description.
    pub description: Option<String>,
}

/// One Lighthouse audit.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LighthouseAudit {
    /// Audit id.
    #[serde(default)]
    pub id: String,
    /// Localized title.
    pub title: Option<String>,
    /// Localized description, often with a link to documentation.
    pub description: Option<String>,
    /// Score from `0.0` to `1.0`. `None` for informative, manual and not-applicable audits.
    pub score: Option<f64>,
    /// How to read the score.
    pub score_display_mode: Option<ScoreDisplayMode>,
    /// Short value for display, such as `2.1 s`.
    pub display_value: Option<String>,
    /// Explanation of a failure.
    pub explanation: Option<String>,
    /// Error message when the audit itself failed.
    pub error_message: Option<String>,
    /// Warnings raised while running the audit.
    pub warnings: Option<Value>,
    /// Raw value, for example milliseconds for timing audits.
    pub numeric_value: Option<f64>,
    /// Unit of [`numeric_value`](Self::numeric_value), such as `millisecond` or `unitless`.
    pub numeric_unit: Option<String>,
    /// Estimated savings per metric, such as `{"LCP": 450}`.
    pub metric_savings: Option<Value>,
    /// Table, opportunity or other details. The shape depends on the audit.
    pub details: Option<Value>,
}

impl LighthouseAudit {
    /// `true` when the audit has a score below `0.9` and is a scored audit.
    /// See [`LighthouseResult::failing_audits`].
    pub fn is_failing(&self) -> bool {
        let scored = matches!(
            self.score_display_mode,
            Some(ScoreDisplayMode::Binary)
                | Some(ScoreDisplayMode::Numeric)
                | Some(ScoreDisplayMode::MetricSavings)
        );
        scored && self.score.is_some_and(|score| score < 0.9)
    }
}

/// How to read [`LighthouseAudit::score`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum ScoreDisplayMode {
    /// Pass (`1`) or fail (`0`).
    Binary,
    /// A score between `0` and `1`.
    Numeric,
    /// Scored by the estimated savings.
    MetricSavings,
    /// Information only, no score.
    Informative,
    /// Must be checked by hand.
    Manual,
    /// Does not apply to this page.
    NotApplicable,
    /// The audit failed to run.
    Error,
    /// A value this crate does not know yet.
    #[serde(other)]
    Unknown,
}
