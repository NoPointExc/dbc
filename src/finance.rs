use serde_json::Value;

/// Fetch the current price of a stock symbol from Yahoo Finance.
/// Uses a blocking HTTP request via `ureq`.
pub fn fetch_stock_price(symbol: &str) -> Result<f64, String> {
    // query2 and v8/chart is often more stable for simple requests
    let url = format!("https://query2.finance.yahoo.com/v8/finance/chart/{}?interval=1m&range=1d", symbol);
    
    let response: Value = ureq::get(&url)
        .set("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .call()
        .map_err(|e| format!("Failed to connect to stock service: {}", e))?
        .into_json()
        .map_err(|e| format!("Failed to parse stock response: {}", e))?;

    let result = response["chart"]["result"]
        .as_array()
        .ok_or_else(|| "Invalid stock response format".to_string())?;

    if result.is_empty() {
        return Err(format!("Stock symbol '{}' not found", symbol));
    }

    let price = result[0]["meta"]["regularMarketPrice"]
        .as_f64()
        .ok_or_else(|| format!("Price data unavailable for '{}'", symbol))?;

    Ok(price)
}
