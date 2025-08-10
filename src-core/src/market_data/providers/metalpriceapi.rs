use std::collections::HashMap;
use serde::Deserialize;
use reqwest::Client;
use async_trait::async_trait;

use crate::market_data::{MarketDataProvider, MarketDataProviderError, Quote};

#[derive(Deserialize)]
struct LatestResponse {
    rates: HashMap<String, f64>,
}

pub struct MetalPriceApi {
    client: Client,
    api_key: String,
}

impl MetalPriceApi {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
    }

    async fn fetch_latest(&self, base: &str, currencies: &[&str]) -> Result<HashMap<String, f64>, reqwest::Error> {
        let currencies_str = currencies.join(",");
        let url = format!(
            "https://api.metalpriceapi.com/v1/latest?api_key={}&base={}&currencies={}",
            self.api_key, base, currencies_str
        );
        let resp = self.client.get(&url).send().await?.json::<LatestResponse>().await?;
        Ok(resp.rates)
    }
}

#[async_trait]
impl MarketDataProvider for MetalPriceApi {
    async fn fetch_quotes(&self, symbols: Vec<String>) -> Result<HashMap<String, Quote>, MarketDataProviderError> {
        let mut needed = Vec::new();
        for s in &symbols {
            match s.as_str() {
                "XAU" | "GOLD" => needed.push("XAU"),
                "XAG" | "SILVER" => needed.push("XAG"),
                "EUR" => needed.push("EUR"),
                _ => {}
            }
        }

        if needed.is_empty() {
            return Ok(HashMap::new());
        }

        let rates = self.fetch_latest("USD", &needed).await
            .map_err(|e| MarketDataProviderError::Other(e.to_string()))?;

        let mut quotes = HashMap::new();
        for (symbol, price) in rates {
            quotes.insert(symbol.clone(), Quote {
                symbol,
                price,
                currency: "USD".to_string(),
                timestamp: None,
            });
        }

        Ok(quotes)
    }
}
