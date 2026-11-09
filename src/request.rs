use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// Device that Lighthouse emulates (`strategy` query parameter).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum Strategy {
    /// Mobile device with a throttled network. This is also the API default.
    #[default]
    Mobile,
    /// Desktop browser.
    Desktop,
}

impl Strategy {
    /// Wire value sent to the API (`MOBILE` / `DESKTOP`).
    pub fn as_str(&self) -> &'static str {
        match self {
            Strategy::Mobile => "MOBILE",
            Strategy::Desktop => "DESKTOP",
        }
    }
}

impl Display for Strategy {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Lighthouse category (`category` query parameter).
///
/// When no category is given the API runs only [`Category::Performance`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum Category {
    /// Performance (lab metrics such as LCP, TBT and CLS).
    Performance,
    /// Accessibility.
    Accessibility,
    /// Best practices.
    BestPractices,
    /// SEO (crawlability, meta tags, structured data basics).
    Seo,
    /// Progressive Web App. Removed from Lighthouse 12, so current runs may not return it.
    Pwa,
    /// Agentic browsing: how well AI agents can use the page.
    AgenticBrowsing,
}

impl Category {
    /// The four standard categories, in the order Lighthouse reports them.
    /// [`Category::Pwa`] and [`Category::AgenticBrowsing`] are not included.
    pub const ALL: [Category; 4] = [
        Category::Performance,
        Category::Accessibility,
        Category::BestPractices,
        Category::Seo,
    ];

    /// Wire value sent to the API (`PERFORMANCE`, `BEST_PRACTICES`, ...).
    pub fn as_str(&self) -> &'static str {
        match self {
            Category::Performance => "PERFORMANCE",
            Category::Accessibility => "ACCESSIBILITY",
            Category::BestPractices => "BEST_PRACTICES",
            Category::Seo => "SEO",
            Category::Pwa => "PWA",
            Category::AgenticBrowsing => "AGENTIC_BROWSING",
        }
    }

    /// Category id used inside `lighthouseResult.categories` (`performance`, `best-practices`, ...).
    pub fn lighthouse_id(&self) -> &'static str {
        match self {
            Category::Performance => "performance",
            Category::Accessibility => "accessibility",
            Category::BestPractices => "best-practices",
            Category::Seo => "seo",
            Category::Pwa => "pwa",
            Category::AgenticBrowsing => "agentic-browsing",
        }
    }
}

impl Display for Category {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Parameters of `pagespeedapi.runpagespeed`.
///
/// # Example
///
/// ```
/// use google_pagespeed_insights_api::{Category, RunPagespeedRequest, Strategy};
///
/// let request = RunPagespeedRequest::new("https://example.com/")
///     .strategy(Strategy::Desktop)
///     .categories([Category::Performance, Category::Seo])
///     .locale("ja");
///
/// assert_eq!(request.url(), "https://example.com/");
/// assert_eq!(request.get_strategy(), Strategy::Desktop);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct RunPagespeedRequest {
    url: String,
    strategy: Strategy,
    categories: Vec<Category>,
    locale: Option<String>,
    utm_campaign: Option<String>,
    utm_source: Option<String>,
    captcha_token: Option<String>,
}

impl RunPagespeedRequest {
    /// Creates a request for `url` with the mobile strategy and the performance category.
    pub fn new<T: Into<String>>(url: T) -> Self {
        RunPagespeedRequest {
            url: url.into(),
            strategy: Strategy::Mobile,
            categories: Vec::new(),
            locale: None,
            utm_campaign: None,
            utm_source: None,
            captcha_token: None,
        }
    }

    /// Sets the emulated device.
    pub fn strategy(mut self, strategy: Strategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// Adds one category to run.
    pub fn category(mut self, category: Category) -> Self {
        if !self.categories.contains(&category) {
            self.categories.push(category);
        }
        self
    }

    /// Adds several categories to run. Use [`Category::ALL`] for every category.
    pub fn categories<I: IntoIterator<Item = Category>>(mut self, categories: I) -> Self {
        for category in categories {
            self = self.category(category);
        }
        self
    }

    /// Sets the language of audit titles and descriptions (for example `ja` or `en`).
    pub fn locale<T: Into<String>>(mut self, locale: T) -> Self {
        self.locale = Some(locale.into());
        self
    }

    /// Sets the `utm_campaign` parameter.
    pub fn utm_campaign<T: Into<String>>(mut self, value: T) -> Self {
        self.utm_campaign = Some(value.into());
        self
    }

    /// Sets the `utm_source` parameter.
    pub fn utm_source<T: Into<String>>(mut self, value: T) -> Self {
        self.utm_source = Some(value.into());
        self
    }

    /// Sets the `captchaToken` parameter, the token from solving a captcha.
    pub fn captcha_token<T: Into<String>>(mut self, value: T) -> Self {
        self.captcha_token = Some(value.into());
        self
    }

    /// The URL to analyze.
    pub fn url(&self) -> &str {
        &self.url
    }

    /// The emulated device.
    pub fn get_strategy(&self) -> Strategy {
        self.strategy
    }

    /// The categories to run. Empty means the API default (performance only).
    pub fn get_categories(&self) -> &[Category] {
        &self.categories
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        let mut pairs = vec![
            ("url", self.url.clone()),
            ("strategy", self.strategy.as_str().to_string()),
        ];
        for category in &self.categories {
            pairs.push(("category", category.as_str().to_string()));
        }
        if let Some(locale) = &self.locale {
            pairs.push(("locale", locale.clone()));
        }
        if let Some(value) = &self.utm_campaign {
            pairs.push(("utm_campaign", value.clone()));
        }
        if let Some(value) = &self.utm_source {
            pairs.push(("utm_source", value.clone()));
        }
        if let Some(value) = &self.captcha_token {
            pairs.push(("captchaToken", value.clone()));
        }
        pairs
    }
}
