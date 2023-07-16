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
    // vector of maps with exchange name as key string and corresponding level
    // for now the entry itself is the usize price entry
    bid_prices_reference: Vec<HashMap<String, Level>>,
    ask_prices_reference: Vec<HashMap<String, Level>>,
    reporting_levels: u32, // to be set as a parmater
    last_update_ids: HashMap<String, u64>,
}

// symbol - crypto pair as String
// levels - number of levels to be monitored as u32 (this is for the summary, the orderbook stores anyway everything it receives)
// ParsedUpdate - struct that cotains 2 vetors of levels (for bids and asks) and a timestamp
impl OrderBook {
    pub fn new(symbol: String, reporting_levels: u32, parsed_update: ParsedUpdate) -> Result<Self> {
        // two random potential values from btcusdt for now
        let best_bid_price: usize = 0;
        let best_ask_price: usize = 30000 as usize;

        // let mut bid_prices_reference: Vec<HashMap<String, Level>> = Vec::with_capacity(best_ask_price as usize * 3);
        // let mut ask_prices_reference: Vec<HashMap<String, Level>> = Vec::with_capacity(best_ask_price as usize * 3);

        // this maybe very space intensive (I don't know, double check)
        // but at least it makes it straightforward and clearer to read and understand
        let bid_prices_reference: Vec<HashMap<String, Level>> =
            vec![HashMap::new(); best_ask_price as usize * 3];
        let ask_prices_reference: Vec<HashMap<String, Level>> =
            vec![HashMap::new(); best_ask_price as usize * 3];

        let mut last_update_ids = HashMap::new(); // to be kept with latest update from each exchange
        last_update_ids.insert("BINANCE".to_string(), 1);
        last_update_ids.insert("BITSTAMP".to_string(), 1);

        let mut order_book = Self {
            symbol: symbol.clone(),
            best_bid_price,
            best_ask_price,
            bid_prices_reference,
            ask_prices_reference,
            reporting_levels,
            last_update_ids,
        };

        // for the moment this kind of logic will keep any update in the reference
        // even when the quantity is set at zero. and just leve it there
        // but this will still need to adjust best_bid/ask_prices
        // possibly looping on what's availabel in teh update

        // refactor into two functions for ask and bid
        // then call them from a method update o the whole ParsedUpdate

        // the loops also need to keep track of both best ask and best bid available
        order_book.merge_parse_update(parsed_update)?;
        Ok(order_book)
    }

    // main method to transform a float price into it's array index equivalent
    fn price_to_price_array_index(&self, price: f64) -> usize {
        let price_index = Decimal::from_f64(price * 100.0).expect("Decimal failed to parse f64");
        price_index.mantissa() as usize
    }

    pub fn merge_parse_update(&mut self, parsed_update: ParsedUpdate) -> Result<()> {
        // this firt checks if for a given exchange we have a last_update_id timestmp
        // higher than current, if not simply returns Ok(())
        if parsed_update.last_update_id
            > *self
                .last_update_ids
                .get(&parsed_update.bids[0].exchange.clone())
                .unwrap()
        {
            self.last_update_ids.insert(
                parsed_update.bids[0].exchange.clone(),
                parsed_update.last_update_id,
            );
        } else {
            return Ok(());
        }

        for bid in parsed_update.bids {
            // parsed_update.last_update_id contains the timestamp to be compared against
            // let exchange = bid.exchange.clone();
            let price_position = self.price_to_price_array_index(bid.price);
            let mut ref_map = self.bid_prices_reference.remove(price_position);
            // put logic here to remove if volume is zero...
            // let exchange = bid.exchange;
            ref_map.insert(bid.exchange.clone(), bid);
            self.bid_prices_reference[price_position] = ref_map;
            if price_position > self.best_bid_price {
                self.best_bid_price = price_position
            }
        }

        for ask in parsed_update.asks {
            let price_position = self.price_to_price_array_index(ask.price);
            let mut ref_map = self.ask_prices_reference.remove(price_position);
            ref_map.insert(ask.exchange.clone(), ask);
            self.ask_prices_reference[price_position] = ref_map;
            if price_position < self.best_ask_price {
                self.best_ask_price = price_position
            }
        }
        return Ok(());
    }

    pub fn merge_bid(level: Level) {
        unimplemented!()
    }

    pub fn merge_ask(level: Level) {
        unimplemented!()
    }

    pub fn get_asks_reporting_levels(&self) -> Result<Vec<Level>> {
        // get top 10 asks
        unimplemented!()
    }

    pub fn get_bids_reporting_levels(&self) -> Result<Vec<Level>> {
        // get top 10 bids
        unimplemented!()
    }

    pub fn get_summary(&self) -> Result<Summary> {
        // get the top 10 summary levels
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_creates_an_orderbook() {
        let symbol = "BTCUSD".to_string();
        let snapshots = ParsedUpdate {
            last_update_id: 100000,
            bids: vec![
                Level {
                    price: 7.0,
                    amount: 1.0,
                    exchange: "BITSTAMP".to_string(),
                },
                Level {
                    price: 8.0,
                    amount: 1.0,
                    exchange: "BITSTAMP".to_string(),
                },
            ],
            asks: vec![Level {
                price: 10.0,
                amount: 1.0,
                exchange: "BITSTAMP".to_string(),
            }],
        };
        let ob = OrderBook::new(symbol, 5, snapshots).unwrap();
        assert_eq!(ob.best_ask_price, 1000);
        assert_eq!(ob.best_bid_price, 800);
    }

    #[test]
    fn always_working() {
        assert_eq!(1, 1);
    }
}
