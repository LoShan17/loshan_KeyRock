use crate::exchanges::ParsedUpdate;
use crate::orderbookaggregator::{Level, Summary};
use anyhow::{Context, Result};
use rust_decimal::{prelude::FromPrimitive, Decimal};
use serde_json::Value;
use std::{
    collections::HashMap,
    ops::{Mul, Sub},
};

#[derive(Debug, Default)]
pub struct OrderBook {
    // The idea is storing price points in a vector for each side of the book
    // Indexing the price with some integer representation (maybe using Decimal)
    // This will allow O(1) retrieval for any price point

    // any lookup integer to retrieve from the main array in O(1)
    // is going to be stored as usize as that seems to be the correct way
    symbol: String,
    best_bid_price: usize,
    best_ask_price: usize,
    bid_prices_reference: Vec<HashMap<String, Level>>,
    ask_prices_reference: Vec<HashMap<String, Level>>,
    reporting_levels: u32,
}

// symbol - crypto pair as String
// levels - number of levels to be monitored as u32 (this is for the summary, the orderbook stores anyway everything it receives)
// ParsedUpdate - struct that cotains 2 vetors of levels (for bids and asks) and a timestamp
impl OrderBook {
    pub fn new(symbol: String, reporting_levels: u32, parsed_update: ParsedUpdate) -> Result<Self> {
        // two random potential values from btcusdt for now
        let best_bid_price: usize = 3033403;
        let best_ask_price: usize = 3033419;

        // let mut bid_prices_reference: Vec<HashMap<String, Level>> = Vec::with_capacity(best_ask_price as usize * 3);
        // let mut ask_prices_reference: Vec<HashMap<String, Level>> = Vec::with_capacity(best_ask_price as usize * 3);

        // this maybe very space intensive (I don't know, double check)
        // but at least it makes it straightforward and clearer to read and understand
        let mut bid_prices_reference: Vec<HashMap<String, Level>> =
            vec![HashMap::new(); best_ask_price as usize * 3];
        let mut ask_prices_reference: Vec<HashMap<String, Level>> =
            vec![HashMap::new(); best_ask_price as usize * 3];

        let mut order_book = Self {
            symbol,
            best_bid_price,
            best_ask_price,
            bid_prices_reference,
            ask_prices_reference,
            reporting_levels,
        };

        // for the moment this kind of logic will keep any update in the reference
        // even when the quantity is set at zero. and just leve it there
        // but this will still need to adjust best_bid/ask_prices
        // possibly looping on what's availabel in teh update

        // the loops also need to keep track of both best ask and best bid available
        for bid in parsed_update.bids {
            let price_position = order_book.price_to_price_array_index(bid.price);
            let mut ref_map = order_book.bid_prices_reference.remove(price_position);
            // put logic here to remove if volume is zero...
            // let exchange = bid.exchange;
            ref_map.insert(bid.exchange.clone(), bid);
            order_book.bid_prices_reference[price_position] = ref_map;
            if price_position > order_book.best_bid_price {
                order_book.best_bid_price = price_position
            }
        }

        for ask in parsed_update.asks {
            let price_position = order_book.price_to_price_array_index(ask.price);
            let mut ref_map = order_book.ask_prices_reference.remove(price_position);
            ref_map.insert(ask.exchange.clone(), ask);
            order_book.bid_prices_reference[price_position] = ref_map;
            if price_position < order_book.best_ask_price {
                order_book.best_ask_price = price_position
            }
        }

        Ok(order_book)
    }

    // main method to transform a float price into it's array index equivalent
    fn price_to_price_array_index(&self, price: f64) -> usize {
        let price_index = Decimal::from_f64(price * 100.0).expect("Decimal failed to parse f64");
        price_index.mantissa() as usize
    }

    pub fn get_asks_reporting_levels(&self) -> Result<Vec<Level>> {
        unimplemented!()
    }

    pub fn get_bids_reporting_levels(&self) -> Result<Vec<Level>> {
        unimplemented!()
    }

    pub fn get_summary(&self) -> Result<Summary> {
        unimplemented!()
    }

    pub fn update(&mut self, parsed_update: &ParsedUpdate) -> Result<()> {
        unimplemented!()
    }
}
