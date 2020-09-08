use crate::error::WalletResult;
use crate::historical::{AssetDay, Historical};
use chrono::{Date, Utc};

impl Historical {
    pub fn get_for_day_with_fallback(symbol: &str, date: Date<Utc>) -> WalletResult<AssetDay> {
        let asset_day = AssetDay {
            symbol: symbol.to_string(),
            time: date.and_hms(13, 0, 0),
            open: 1.0,
            high: 15.0,
            low: 0.5,
            close: 9.0,
            volume: 100,
        };

        Ok(asset_day)
    }

    #[tokio::main]
    pub async fn current_price_for_symbol(_symbol: String) -> f64 {
        9.0
    }
}
