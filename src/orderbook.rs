use crate::orderbookaggregator::{Level, Summary};
use std::collections::HashMap;
use anyhow::{Context, Result};
use serde_json::Value;
use rust_decimal::{Decimal, prelude::FromPrimitive};

#[derive(Debug, Default)]
pub struct OrderBook {
    // The idea is storing price points in a vector for each side of the book
    // Indexing the price with some integer representation (maybe using Decimal)
    // This will allow O(1) retrieval for any price point
    symbol: String,
    best_bid_price: u32,
    best_ask_price: u32,
    bid_prices_reference: Vec<HashMap<&'static str, Level>>,
    ask_prices_reference: Vec<HashMap<&'static str, Level>>
}

impl OrderBook {

    pub fn new(symbol: String, levels: u32, snapshots: Vec<Value>) -> Result<Self> {

        let best_bid_price: u32 = 10;
        let best_ask_price: u32 = 12;
        
        let snapshot = &snapshots[0];
        let one_level = Level {
            price: 10.00,
            amount: 0.5,
            exchange: "BITSTAMP".to_string(),
        };


        let two_level = Level {
            price: 10.00,
            amount: 0.5,
            exchange: "BINANCE".to_string(),
        };

        let mut level_map_1 = HashMap::new();
        level_map_1.insert("BITSTAMP", one_level).unwrap();

        let mut level_map_2 = HashMap::new();
        level_map_2.insert("BINANCE", two_level).unwrap();

        let mut bid_prices_reference = vec![level_map_1];
        let mut ask_prices_reference = vec![level_map_2];

        let mut order_book = Self {
            symbol,
            best_bid_price,
            best_ask_price,
            bid_prices_reference,
            ask_prices_reference    
        };

        Ok(order_book)
    }

    pub fn get_asks_levels() -> Result<Vec<Level>> {
        unimplemented!()
    }

    pub fn get_bids_levels() -> Result<Vec<Level>> {
        unimplemented!()   
    }

    pub fn get_summary(&self) -> Result<Summary> {
        unimplemented!()

    }
}