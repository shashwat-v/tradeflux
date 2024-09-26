use yahoo_finance_api as yahoo;
use ndarray::{Array1, s};
use std::error::Error;
use tokio;

/// Function to calculate the moving average
fn moving_average(data: &Array1<f64>, window: usize) -> Array1<f64> {
    let mut ma = Array1::<f64>::zeros(data.len());

    for i in window..data.len() {
        let sum: f64 = data.slice(s![i - window..i]).sum();
        ma[i] = sum / window as f64;
    }
    ma
}

/// Function to calculate RSI
fn calculate_rsi(data: &Array1<f64>, period: usize) -> Array1<f64> {
    let mut rsi = Array1::<f64>::zeros(data.len());
    let mut gain = 0.0;
    let mut loss = 0.0;

    // Initialize the first period gains and losses
    for i in 1..=period {
        let change = data[i] - data[i - 1];
        if change > 0.0 {
            gain += change;
        } else {
            loss -= change;
        }
    }

    let mut avg_gain = gain / period as f64;
    let mut avg_loss = loss / period as f64;
    rsi[period] = 100.0 - (100.0 / (1.0 + avg_gain / avg_loss));

    // Calculate RSI for the remaining data points
    for i in (period + 1)..data.len() {
        let change = data[i] - data[i - 1];
        if change > 0.0 {
            avg_gain = (avg_gain * (period as f64 - 1.0) + change) / period as f64;
            avg_loss = (avg_loss * (period as f64 - 1.0)) / period as f64;
        } else {
            avg_gain = (avg_gain * (period as f64 - 1.0)) / period as f64;
            avg_loss = (avg_loss * (period as f64 - 1.0) - change) / period as f64;
        }

        rsi[i] = 100.0 - (100.0 / (1.0 + avg_gain / avg_loss));
    }

    rsi
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Fetch stock data using yahoo_finance_api
    let provider = yahoo::YahooConnector::new().expect("Failed to create YahooConnector");
    
    // Fetch historical data for the past 6 months
    let response = provider.get_quote_range("AAPL", "1d", "4y").await?;
    
    // Extract close prices from the response
    let quotes = response.quotes().unwrap();
    let close_prices: Vec<f64> = quotes.iter().map(|q| q.close).collect();

    if close_prices.len() < 200 {
        println!("Not enough data points for moving average calculations.");
        return Ok(());
    }

    let close_array = Array1::from(close_prices);

    // Calculate the 50-day and 200-day moving averages
    let ma50 = moving_average(&close_array, 50);
    let ma200 = moving_average(&close_array, 200);

    // Calculate the RSI with a 14-period window
    let rsi = calculate_rsi(&close_array, 14);

    // Identify Buy, Sell, Hold signals based on Golden Cross and Death Cross
    let mut signals: Vec<&str> = vec!["Hold"; close_array.len()];
    for i in 1..close_array.len() {
        if i < 200 {
            continue;
        }

        if ma50[i] > ma200[i] && ma50[i - 1] <= ma200[i - 1] && rsi[i] < 70.0 {
            signals[i] = "Buy";
        } else if ma50[i] < ma200[i] && ma50[i - 1] >= ma200[i - 1] && rsi[i] > 30.0 {
            signals[i] = "Sell";
        }
    }

    // Display the buy/sell signals
    for (i, signal) in signals.iter().enumerate() {
        if *signal != "Hold" {
            println!("Day {}: Signal = {}", i, signal);
        }
    }

    Ok(())
}
