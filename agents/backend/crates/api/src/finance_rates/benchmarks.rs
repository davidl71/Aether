//! FMP, FRED, and New York Fed benchmark fetchers.

use crate::finance_rates::types::{
    BenchmarkRate, BenchmarkRateResponse, SofrBenchmarksResponse, SofrOvernightResponse,
    TreasuryBenchmarksResponse,
};
use chrono::Utc;
use market_data::FmpClient;
use reqwest::Client;
use serde_json::Value;

const FRED_BASE_URL: &str = "https://api.stlouisfed.org/fred";
const NEW_YORK_FED_BASE_URL: &str = "https://markets.newyorkfed.org/api";

pub async fn get_sofr_rates(client: &Client) -> SofrBenchmarksResponse {
    let overnight = fetch_sofr_overnight(client).await;
    let term_rates = fetch_sofr_term_rates(client).await;

    SofrBenchmarksResponse {
        overnight: SofrOvernightResponse {
            rate: overnight.as_ref().map(|rate| rate.rate),
            timestamp: overnight.as_ref().map(|rate| rate.timestamp.clone()),
        },
        term_rates: term_rates
            .into_iter()
            .map(BenchmarkRateResponse::from)
            .collect(),
        timestamp: Utc::now().to_rfc3339(),
    }
}

pub async fn get_treasury_rates(client: &Client) -> TreasuryBenchmarksResponse {
    let rates = fetch_treasury_rates(client).await;
    TreasuryBenchmarksResponse {
        rates: rates.into_iter().map(BenchmarkRateResponse::from).collect(),
        timestamp: Utc::now().to_rfc3339(),
    }
}

pub(crate) async fn all_benchmarks(client: &Client) -> Vec<BenchmarkRate> {
    let mut benchmarks = Vec::new();
    if let Some(overnight) = fetch_sofr_overnight(client).await {
        benchmarks.push(overnight);
    }
    benchmarks.extend(fetch_sofr_term_rates(client).await);
    benchmarks.extend(fetch_treasury_rates(client).await);
    benchmarks
}

pub(crate) async fn fetch_sofr_overnight(client: &Client) -> Option<BenchmarkRate> {
    if let Some(rate) = fetch_sofr_overnight_from_fmp().await {
        return Some(rate);
    }

    if let Some(api_key) = fred_api_key() {
        if let Some(rate) = fetch_fred_latest_series(client, "SOFR", Some(&api_key), 1).await {
            return Some(BenchmarkRate {
                rate_type: "SOFR".to_string(),
                tenor: "Overnight".to_string(),
                days_to_expiry: Some(1),
                rate: rate.0,
                timestamp: Utc::now().to_rfc3339(),
                source: "FRED (St. Louis Fed)".to_string(),
            });
        }
    }

    let url = format!("{NEW_YORK_FED_BASE_URL}/rates/all");
    let Ok(response) = client.get(url).send().await else {
        return None;
    };
    let Ok(json) = response.json::<Value>().await else {
        return None;
    };
    let sofr = json
        .get("sofr")
        .or_else(|| json.get("SOFR"))
        .and_then(Value::as_object)?;
    let rate = sofr
        .get("rate")
        .or_else(|| sofr.get("value"))
        .and_then(Value::as_f64)?;
    if rate <= 0.0 {
        return None;
    }
    Some(BenchmarkRate {
        rate_type: "SOFR".to_string(),
        tenor: "Overnight".to_string(),
        days_to_expiry: Some(1),
        rate,
        timestamp: Utc::now().to_rfc3339(),
        source: "New York Fed".to_string(),
    })
}

async fn fetch_sofr_overnight_from_fmp() -> Option<BenchmarkRate> {
    let api_key = crate::credentials::fmp_api_key()?;
    let client = FmpClient::new(api_key, None).ok()?;
    let rates = client.sofr_rates().await.ok()?;
    let latest = rates.into_iter().next()?;
    let rate = latest.rate? / 100.0;
    Some(BenchmarkRate {
        rate_type: "SOFR".to_string(),
        tenor: "Overnight".to_string(),
        days_to_expiry: Some(1),
        rate,
        timestamp: Utc::now().to_rfc3339(),
        source: "FMP".to_string(),
    })
}

pub(crate) async fn fetch_sofr_term_rates(client: &Client) -> Vec<BenchmarkRate> {
    let Some(api_key) = fred_api_key() else {
        return Vec::new();
    };

    let mut rates = Vec::new();
    for (tenor, series_id, dte) in [
        ("1M", "SOFR30DAYAVG", 30),
        ("3M", "SOFR90DAYAVG", 90),
        ("6M", "SOFR180DAYAVG", 180),
    ] {
        if let Some((rate, _date)) =
            fetch_fred_latest_series(client, series_id, Some(&api_key), 1).await
        {
            rates.push(BenchmarkRate {
                rate_type: "SOFR".to_string(),
                tenor: tenor.to_string(),
                days_to_expiry: Some(dte),
                rate,
                timestamp: Utc::now().to_rfc3339(),
                source: "FRED (St. Louis Fed)".to_string(),
            });
        }
    }

    if let Some(rate) = fetch_sofr_one_year(client, &api_key).await {
        rates.push(rate);
    }

    rates
}

async fn fetch_sofr_one_year(client: &Client, api_key: &str) -> Option<BenchmarkRate> {
    let observations = fetch_fred_series_observations(client, "SOFRINDEX", api_key, 400).await?;
    if observations.len() <= 252 {
        return None;
    }
    let latest = &observations[0];
    let year_ago = &observations[252];
    let latest_value = latest.1?;
    let year_ago_value = year_ago.1?;
    if latest_value <= 0.0 || year_ago_value <= 0.0 {
        return None;
    }
    let rate = (latest_value / year_ago_value - 1.0) * 100.0;
    if !(0.1..25.0).contains(&rate) {
        return None;
    }
    Some(BenchmarkRate {
        rate_type: "SOFR".to_string(),
        tenor: "1Y".to_string(),
        days_to_expiry: Some(365),
        rate,
        timestamp: Utc::now().to_rfc3339(),
        source: "FRED (St. Louis Fed)".to_string(),
    })
}

pub(crate) async fn fetch_treasury_rates(client: &Client) -> Vec<BenchmarkRate> {
    if let Some(rates) = fetch_treasury_rates_from_fmp().await {
        return rates;
    }

    let Some(api_key) = fred_api_key() else {
        return Vec::new();
    };

    let mut rates = Vec::new();
    for (tenor, series_id, dte) in [
        ("1M", "DGS1MO", 30),
        ("3M", "DGS3MO", 90),
        ("6M", "DGS6MO", 180),
        ("1Y", "DGS1", 365),
        ("2Y", "DGS2", 730),
        ("5Y", "DGS5", 1825),
        ("10Y", "DGS10", 3650),
        ("30Y", "DGS30", 10950),
    ] {
        if let Some((rate, _date)) =
            fetch_fred_latest_series(client, series_id, Some(&api_key), 1).await
        {
            rates.push(BenchmarkRate {
                rate_type: "Treasury".to_string(),
                tenor: tenor.to_string(),
                days_to_expiry: Some(dte),
                rate,
                timestamp: Utc::now().to_rfc3339(),
                source: "FRED (St. Louis Fed)".to_string(),
            });
        }
    }
    rates
}

async fn fetch_treasury_rates_from_fmp() -> Option<Vec<BenchmarkRate>> {
    let api_key = crate::credentials::fmp_api_key()?;
    let client = FmpClient::new(api_key, None).ok()?;
    let rate = client.treasury_rates().await.ok()?;

    let mut rates = Vec::new();
    let timestamp = Utc::now().to_rfc3339();

    if let Some(r) = rate.one_month {
        rates.push(BenchmarkRate {
            rate_type: "Treasury".to_string(),
            tenor: "1M".to_string(),
            days_to_expiry: Some(30),
            rate: r / 100.0,
            timestamp: timestamp.clone(),
            source: "FMP".to_string(),
        });
    }
    if let Some(r) = rate.three_month {
        rates.push(BenchmarkRate {
            rate_type: "Treasury".to_string(),
            tenor: "3M".to_string(),
            days_to_expiry: Some(90),
            rate: r / 100.0,
            timestamp: timestamp.clone(),
            source: "FMP".to_string(),
        });
    }
    if let Some(r) = rate.six_month {
        rates.push(BenchmarkRate {
            rate_type: "Treasury".to_string(),
            tenor: "6M".to_string(),
            days_to_expiry: Some(180),
            rate: r / 100.0,
            timestamp: timestamp.clone(),
            source: "FMP".to_string(),
        });
    }
    if let Some(r) = rate.one_year {
        rates.push(BenchmarkRate {
            rate_type: "Treasury".to_string(),
            tenor: "1Y".to_string(),
            days_to_expiry: Some(365),
            rate: r / 100.0,
            timestamp: timestamp.clone(),
            source: "FMP".to_string(),
        });
    }
    if let Some(r) = rate.two_year {
        rates.push(BenchmarkRate {
            rate_type: "Treasury".to_string(),
            tenor: "2Y".to_string(),
            days_to_expiry: Some(730),
            rate: r / 100.0,
            timestamp: timestamp.clone(),
            source: "FMP".to_string(),
        });
    }
    if let Some(r) = rate.five_year {
        rates.push(BenchmarkRate {
            rate_type: "Treasury".to_string(),
            tenor: "5Y".to_string(),
            days_to_expiry: Some(1825),
            rate: r / 100.0,
            timestamp: timestamp.clone(),
            source: "FMP".to_string(),
        });
    }
    if let Some(r) = rate.ten_year {
        rates.push(BenchmarkRate {
            rate_type: "Treasury".to_string(),
            tenor: "10Y".to_string(),
            days_to_expiry: Some(3650),
            rate: r / 100.0,
            timestamp: timestamp.clone(),
            source: "FMP".to_string(),
        });
    }
    if let Some(r) = rate.thirty_year {
        rates.push(BenchmarkRate {
            rate_type: "Treasury".to_string(),
            tenor: "30Y".to_string(),
            days_to_expiry: Some(10950),
            rate: r / 100.0,
            timestamp: timestamp.clone(),
            source: "FMP".to_string(),
        });
    }

    if rates.is_empty() {
        return None;
    }
    Some(rates)
}

async fn fetch_fred_latest_series(
    client: &Client,
    series_id: &str,
    api_key: Option<&str>,
    limit: usize,
) -> Option<(f64, String)> {
    let api_key = api_key?;
    let observations = fetch_fred_series_observations(client, series_id, api_key, limit).await?;
    observations
        .into_iter()
        .find_map(|(date, value)| value.map(|rate| (rate, date)))
}

async fn fetch_fred_series_observations(
    client: &Client,
    series_id: &str,
    api_key: &str,
    limit: usize,
) -> Option<Vec<(String, Option<f64>)>> {
    let url = format!("{FRED_BASE_URL}/series/observations");
    let response = client
        .get(url)
        .query(&[
            ("series_id", series_id),
            ("api_key", api_key),
            ("file_type", "json"),
            ("limit", &limit.to_string()),
            ("sort_order", "desc"),
        ])
        .send()
        .await
        .ok()?;
    let json = response.json::<Value>().await.ok()?;
    let observations = json.get("observations")?.as_array()?;
    Some(
        observations
            .iter()
            .filter_map(|item| {
                let date = item.get("date")?.as_str()?.to_string();
                let value = match item.get("value")?.as_str()? {
                    "." => None,
                    raw => raw.parse::<f64>().ok(),
                };
                Some((date, value))
            })
            .collect(),
    )
}

fn fred_api_key() -> Option<String> {
    crate::credentials::fred_api_key()
}

impl From<BenchmarkRate> for BenchmarkRateResponse {
    fn from(value: BenchmarkRate) -> Self {
        Self {
            tenor: value.tenor,
            rate: value.rate,
            days_to_expiry: value.days_to_expiry,
            timestamp: value.timestamp,
            source: value.source,
        }
    }
}
