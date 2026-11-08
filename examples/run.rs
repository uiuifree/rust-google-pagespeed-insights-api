//! Runs PageSpeed Insights for one URL and prints the scores, Core Web Vitals
//! and failing audits.
//!
//! ```bash
//! PAGESPEED_API_KEY=... cargo run --example run -- https://example.com/ [desktop]
//! ```

use google_pagespeed_insights_api::{
    Category, PageSpeedInsightsApi, RunPagespeedRequest, Strategy,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let url = args.next().ok_or("usage: run <url> [mobile|desktop]")?;
    let strategy = match args.next().as_deref() {
        Some("desktop") => Strategy::Desktop,
        _ => Strategy::Mobile,
    };

    let api = match std::env::var("PAGESPEED_API_KEY") {
        Ok(key) => PageSpeedInsightsApi::with_api_key(key),
        Err(_) => PageSpeedInsightsApi::new(),
    };
    let request = RunPagespeedRequest::new(url)
        .strategy(strategy)
        .categories(Category::ALL);
    let response = api.run_pagespeed(&request).await?;

    println!("analyzed: {}", response.id);
    for category in Category::ALL {
        if let Some(score) = response.category_score(category) {
            println!("{:>15}: {:.0}", category.lighthouse_id(), score * 100.0);
        }
    }

    match response.page_field_summary() {
        Some(field) => println!("field (page): {}", serde_json::to_string(&field)?),
        None => println!("field (page): no data"),
    }
    if let Some(field) = response.origin_field_summary() {
        println!("field (origin): {}", serde_json::to_string(&field)?);
    }
    if let Some(lab) = response.lab_summary() {
        println!("lab: {}", serde_json::to_string(&lab)?);
    }

    if let Some(lighthouse) = &response.lighthouse_result {
        for category in Category::ALL {
            for audit in lighthouse.failing_audits(category) {
                println!(
                    "[{}] {} {} {}",
                    category.lighthouse_id(),
                    audit.id,
                    audit.score.unwrap_or_default(),
                    audit.display_value.as_deref().unwrap_or("")
                );
            }
        }
    }
    Ok(())
}
