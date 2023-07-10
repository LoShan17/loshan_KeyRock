use crate::orderbookaggregator::{Level, Summary};
use std::collections::HashMap;
use anyhow::{Context, Result};
use serde_json::Value;

#[derive(Debug, Default)]
pub struct OrderBook {
    symbol: String,
    best_bid_price: u64,
    best_ask_price: u64,
    // Order id -> (Side, Price_level)
    // prices = Vec<>,
    price_references: Vec<HashMap<&'static str, Level>>
}

impl OrderBook {

    pub fn new(symbol: String, levels: u32, snapshots: Vec<Value>) -> Result<Self> {

        let best_bid_price: u64 = 10;
        let best_ask_price: u64 = 12;
        

        let one_level = Level {
            price: 10.00,
            amount: 0.5,
            exchange: "BITSTAMP".to_string(),
        };

        let mut level_map = HashMap::new();
        level_map.insert("BITSTAMP", one_level).unwrap();

        let mut price_references = vec![level_map];

        let mut order_book = Self {
            symbol,
            best_bid_price,
            best_ask_price,
            price_references      
        };

        Ok(order_book)
    }

    pub fn get_asks_levels() -> Vec<Level> {
        unimplemented!()
    }

    pub fn get_bids_levels() -> Vec<Level> {
        unimplemented!()   
    }

    pub fn get_summary(&self) -> Result<Summary> {
        unimplemented!()
        // let summary_asks = self.get_asks_levels();
        // let summary_bids = self.get_bids_levels();

        // if summary_asks.is_empty() || summary_bids.is_empty() {
        //     tracing::error!(
        //         "Summary spread cannot be calculated with {} bids and {} asks",
        //         summary_bids.len(),
        //         summary_asks.len()
        //     );
        //     bail!("Summary spread cannot be calculated".to_string());
        // }

        // Ok(Summary {
        //     spread: summary_asks[0].price - summary_bids[0].price,
        //     bids: summary_bids,
        //     asks: summary_asks,
        // })
    }
}