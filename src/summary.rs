//! Flat summaries of the numbers people look at first.

use crate::response::{
    FieldCategory, FieldMetricName, LighthouseResult, LoadingExperience, PagespeedResponse,
};
use serde::{Deserialize, Serialize};

/// Core Web Vitals and related field metrics at the 75th percentile.
///
/// Built from a [`LoadingExperience`]. Times are in milliseconds and CLS is
/// the plain score (for example `0.05`), already divided by 100.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FieldSummary {
    /// The URL or origin the data belongs to.
    pub id: Option<String>,
    /// `true` when the data is origin data shown for a URL without enough traffic.
    pub origin_fallback: bool,
    /// Overall Core Web Vitals assessment.
    pub overall: Option<FieldCategory>,
    /// Largest Contentful Paint.
    pub lcp: Option<FieldValue>,
    /// Interaction to Next Paint.
    pub inp: Option<FieldValue>,
    /// Cumulative Layout Shift.
    pub cls: Option<FieldValue>,
    /// First Contentful Paint.
    pub fcp: Option<FieldValue>,
    /// Time to First Byte.
    pub ttfb: Option<FieldValue>,
}

/// A field metric value and its assessment.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FieldValue {
    /// 75th percentile.
    pub p75: f64,
    /// Assessment of the 75th percentile.
    pub category: Option<FieldCategory>,
}

impl FieldSummary {
    /// Builds a summary from field data. Returns `None` when there are no metrics.
    ///
    /// # Example
    ///
    /// ```
    /// use google_pagespeed_insights_api::{FieldSummary, LoadingExperience};
    ///
    /// let data: LoadingExperience = serde_json::from_str(r#"{
    ///   "id": "https://example.com/",
    ///   "metrics": {
    ///     "CUMULATIVE_LAYOUT_SHIFT_SCORE": {"percentile": 5, "category": "FAST"},
    ///     "LARGEST_CONTENTFUL_PAINT_MS": {"percentile": 2100, "category": "FAST"}
    ///   },
    ///   "overall_category": "FAST"
    /// }"#).unwrap();
    ///
    /// let summary = FieldSummary::from_loading_experience(&data).unwrap();
    /// assert_eq!(summary.lcp.unwrap().p75, 2100.0);
    /// assert_eq!(summary.cls.unwrap().p75, 0.05);
    /// assert!(summary.inp.is_none());
    /// ```
    pub fn from_loading_experience(data: &LoadingExperience) -> Option<FieldSummary> {
        if !data.has_data() {
            return None;
        }
        let value = |name: FieldMetricName, scale: f64| {
            let metric = data.metric(name)?;
            Some(FieldValue {
                p75: metric.percentile? / scale,
                category: metric.category,
            })
        };
        Some(FieldSummary {
            id: data.id.clone(),
            origin_fallback: data.origin_fallback.unwrap_or(false),
            overall: data.overall_category,
            lcp: value(FieldMetricName::LargestContentfulPaint, 1.0),
            inp: value(FieldMetricName::InteractionToNextPaint, 1.0),
            cls: value(FieldMetricName::CumulativeLayoutShift, 100.0),
            fcp: value(FieldMetricName::FirstContentfulPaint, 1.0),
            ttfb: value(FieldMetricName::TimeToFirstByte, 1.0),
        })
    }
}

/// Lighthouse lab metrics of one run.
///
/// Times are in milliseconds. Each value is `None` when the audit is missing,
/// for example when the performance category was not requested.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LabSummary {
    /// Performance score from `0.0` to `1.0`.
    pub performance_score: Option<f64>,
    /// First Contentful Paint.
    pub fcp: Option<f64>,
    /// Largest Contentful Paint.
    pub lcp: Option<f64>,
    /// Total Blocking Time.
    pub tbt: Option<f64>,
    /// Cumulative Layout Shift.
    pub cls: Option<f64>,
    /// Speed Index.
    pub speed_index: Option<f64>,
    /// Server response time of the main document.
    pub server_response_time: Option<f64>,
}

impl LabSummary {
    /// Builds a summary from a Lighthouse result.
    pub fn from_lighthouse(result: &LighthouseResult) -> LabSummary {
        let value = |id: &str| result.audit(id).and_then(|a| a.numeric_value);
        LabSummary {
            performance_score: result.category_score(crate::Category::Performance),
            fcp: value("first-contentful-paint"),
            lcp: value("largest-contentful-paint"),
            tbt: value("total-blocking-time"),
            cls: value("cumulative-layout-shift"),
            speed_index: value("speed-index"),
            server_response_time: value("server-response-time"),
        }
    }
}

impl PagespeedResponse {
    /// Field data summary for the URL. See [`FieldSummary`].
    pub fn page_field_summary(&self) -> Option<FieldSummary> {
        FieldSummary::from_loading_experience(self.loading_experience.as_ref()?)
    }

    /// Field data summary for the origin. See [`FieldSummary`].
    pub fn origin_field_summary(&self) -> Option<FieldSummary> {
        FieldSummary::from_loading_experience(self.origin_loading_experience.as_ref()?)
    }

    /// Lab metrics summary. See [`LabSummary`].
    pub fn lab_summary(&self) -> Option<LabSummary> {
        Some(LabSummary::from_lighthouse(
            self.lighthouse_result.as_ref()?,
        ))
    }
}
