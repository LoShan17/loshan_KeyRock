use crate::exchanges::ParsedUpdate;
use crate::orderbookaggregator::{Level, Summary};
use anyhow::Result; // {Context,
use rust_decimal::{prelude::FromPrimitive, Decimal};
// use serde_json::Value;
use std::collections::BTreeMap;
use std::collections::HashMap;

// main method to transform a float price into it's array index equivalent
pub fn price_to_price_map_index(price: f64) -> usize {
    let price_index =
        Decimal::from_f64(price * 100.0).expect("Decimal failed to parse f64 for price");
    return price_index.mantissa() as usize;
}

pub fn volume_to_volume_mantissa(volume: f64) -> u32 {
    let price_index = Decimal::from_f64(volume).expect("Decimal failed to parse f64 for volume");
    return price_index.mantissa() as u32;
}

#[derive(Debug, Default)]
pub struct OrderBook {
    // The idea is storing price points in a BTreeMap
    // Indexing the price with some integer representation (maybe using Decimal)
    // This will allow O(1) retrieval for any price point
    // also the BTreeMap is ideal for sice of the price refence and keep tha data sorted

    // any lookup integer to retrieve from the main array in O(1)
    // is going to be stored as usize
    pub best_bid_price: usize,
    pub best_ask_price: usize,
    // vector of maps with exchange name as key string and corresponding level
    // for now the entry itself is the usize price entry, chnaging this into hash maps of hash maps?
    bid_prices_reference: BTreeMap<usize, HashMap<String, Level>>,
    ask_prices_reference: BTreeMap<usize, HashMap<String, Level>>,
    pub reporting_levels: u32, // to be set as a parmater
    pub last_update_ids: HashMap<String, u64>,
}

// levels - number of levels to be monitored as u32 (this is for the summary, the orderbook stores anyway everything it receives)
// ParsedUpdate - struct that cotains 2 vetors of levels (for bids and asks) and a timestamp
impl OrderBook {
    pub fn new(reporting_levels: u32, parsed_update: ParsedUpdate) -> Result<Self> {
        // two random potential values from btcusdt for now
        let best_bid_price: usize = 0;
        let best_ask_price: usize = usize::MAX; // random starting value, remember to change this

        // let mut bid_prices_reference: Vec<HashMap<String, Level>> = Vec::with_capacity(best_ask_price as usize * 3);
        // let mut ask_prices_reference: Vec<HashMap<String, Level>> = Vec::with_capacity(best_ask_price as usize * 3);

        // this maybe very space intensive (I don't know, double check)
        // but at least it makes it straightforward and clearer to read and understand
        // this thing is horribly ineficient -> change to collections::BTreeMapCopy, which represents a sorted Map
        let bid_prices_reference: BTreeMap<usize, HashMap<String, Level>> = BTreeMap::new();
        let ask_prices_reference: BTreeMap<usize, HashMap<String, Level>> = BTreeMap::new();

        let mut last_update_ids = HashMap::new(); // to be kept with latest update from each exchange
        last_update_ids.insert("BINANCE".to_string(), 1);
        last_update_ids.insert("BITSTAMP".to_string(), 1);

        let mut order_book = Self {
            best_bid_price,
            best_ask_price,
            bid_prices_reference,
            ask_prices_reference,
            reporting_levels,
            last_update_ids,
        };

        // for the moment this kind of logic will keep any Level Map
        // even when the quantity is set at zero. and just leve it there
        order_book.merge_parse_update(parsed_update)?;
        Ok(order_book)
    }

    pub fn merge_parse_update(&mut self, parsed_update: ParsedUpdate) -> Result<()> {
        // this firt checks if for a given exchange we have a last_update_id timestmp
        // higher than current, if not simply returns Ok(())

        let exchange_identifier;
        if parsed_update.bids.len() >= 1 {
            exchange_identifier = parsed_update.bids[0].exchange.clone();
        }
        else {
            exchange_identifier = parsed_update.asks[0].exchange.clone();
        }
        
        // find a better way to identify exchange instead of hardcoding bids[0]
        if parsed_update.last_update_id
            > *self
                .last_update_ids
                .get(&exchange_identifier)
                .expect("failed to retrieve last_update_timestamp for exchange")
        {
            self.last_update_ids.insert(
                exchange_identifier,
                parsed_update.last_update_id,
            );
        } else {
            return Ok(());
        }

        // sort these two loops below? worth it?
        for bid in parsed_update.bids {
            self.merge_bid(bid)?
        }

        for ask in parsed_update.asks {
            self.merge_ask(ask)?
        }

        return Ok(());
    }

    pub fn merge_bid(&mut self, level: Level) -> Result<()> {
        let price_position = price_to_price_map_index(level.price);
        let ref_map = self
            .bid_prices_reference
            .entry(price_position)
            .or_insert(HashMap::new());

        if volume_to_volume_mantissa(level.amount) == 0 {
            println!("catched volume as zeroooo for bid!!!!");
            ref_map.remove(&level.exchange);
            if (price_position == self.best_bid_price) && (ref_map.len() == 0) {
                for (next_price_index, next_exchange_map) in self.bid_prices_reference.iter().rev()
                {
                    if next_exchange_map.len() > 0 {
                        self.best_bid_price = *next_price_index;
                        break;
                    }
                }
            }
        } else {
            ref_map.insert(level.exchange.clone(), level);
            if price_position > self.best_bid_price {
                self.best_bid_price = price_position
            }
        }
        // this is still useless
        // self.ask_prices_reference.insert(price_position, *ref_map);
        return Ok(());
    }

    pub fn merge_ask(&mut self, level: Level) -> Result<()> {
        let price_position = price_to_price_map_index(level.price);
        let ref_map = self
            .ask_prices_reference
            .entry(price_position)
            .or_insert(HashMap::new());

        if volume_to_volume_mantissa(level.amount) == 0 {
            println!("catched volume as zeroooo for ask!!!!");
            ref_map.remove(&level.exchange);
            if (price_position == self.best_ask_price) && (ref_map.len() == 0) {
                for (next_price_index, next_exchange_map) in self.ask_prices_reference.iter() {
                    if next_exchange_map.len() > 0 {
                        self.best_ask_price = *next_price_index;
                        break;
                    }
                }
            }
        } else {
            ref_map.insert(level.exchange.clone(), level);
            if price_position < self.best_ask_price {
                self.best_ask_price = price_position
            }
        }
        return Ok(());
    }

    pub fn get_asks_reporting_levels(&mut self) -> Result<Vec<Level>> {
        let mut selected_ask: Vec<Level> = Vec::new();
        let mut count = 0;

        for (_, exchange_levels_map) in self.ask_prices_reference.iter() {
            for (_, level) in exchange_levels_map {
                selected_ask.push(level.clone());
                count += 1;
                if count == self.reporting_levels {
                    break;
                }
            }
            if count == self.reporting_levels {
                break;
            }
        }
        return Ok(selected_ask);
    }

    pub fn get_bids_reporting_levels(&mut self) -> Result<Vec<Level>> {
        let mut selected_bids: Vec<Level> = Vec::new();
        let mut count = 0;

        // bids should be iterated from larger to smaller so .rev()
        for (_, exchange_levels_map) in self.bid_prices_reference.iter().rev() {
            for (_, level) in exchange_levels_map {
                selected_bids.push(level.clone());
                count += 1;
                if count == self.reporting_levels {
                    break;
                }
            }
            if count == self.reporting_levels {
                break;
            }
        }
        return Ok(selected_bids);
    }

    pub fn get_summary(&mut self) -> Result<Summary> {
        let bids = self.get_bids_reporting_levels()?;
        let asks = self.get_asks_reporting_levels()?;
        return Ok(Summary {
            spread: (self.best_ask_price as f64 / 100.0 - self.best_bid_price as f64 / 100.0),
            bids,
            asks,
        });
    }
}

// Tests start here
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_an_orderbook() {
        let snapshots = ParsedUpdate {
            last_update_id: 100000,
            bids: vec![Level {
                price: 8.0,
                amount: 1.0,
                exchange: "BITSTAMP".to_string(),
            }],
            asks: vec![Level {
                price: 10.0,
                amount: 1.0,
                exchange: "BITSTAMP".to_string(),
            }],
        };
        let ob = OrderBook::new(5, snapshots).unwrap();
        assert_eq!(ob.best_ask_price, 1000);
        assert_eq!(ob.best_bid_price, 800);
    }

    #[test]
    fn creates_an_orderbook_and_deletes_best_bid_ask() {
        let snapshots = ParsedUpdate {
            last_update_id: 100000, // make it newer update
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
            asks: vec![
                Level {
                    price: 10.0,
                    amount: 1.0,
                    exchange: "BITSTAMP".to_string(),
                },
                Level {
                    price: 11.00,
                    amount: 1.0,
                    exchange: "BITSTAMP".to_string(),
                },
            ],
        };
        let mut ob = OrderBook::new(5, snapshots).unwrap();
        let new_update = ParsedUpdate {
            last_update_id: 110000,
            bids: vec![Level {
                price: 8.0,
                amount: 0.0,
                exchange: "BITSTAMP".to_string(),
            }],
            asks: vec![Level {
                price: 10.0,
                amount: 0.0,
                exchange: "BITSTAMP".to_string(),
            }],
        };
        ob.merge_parse_update(new_update)
            .expect("broken merge update");
        assert_eq!(ob.best_bid_price, 700);
        assert_eq!(ob.best_ask_price, 1100);
    }

    #[test]
    fn creates_an_orderbook_and_adds_best_bid() {
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
        let mut ob = OrderBook::new(5, snapshots).unwrap();
        let new_update = ParsedUpdate {
            last_update_id: 110000,
            bids: vec![Level {
                price: 9.0,
                amount: 1.0,
                exchange: "BITSTAMP".to_string(),
            }],
            asks: vec![Level {
                price: 11.0,
                amount: 1.0,
                exchange: "BITSTAMP".to_string(),
            }],
        };
        ob.merge_parse_update(new_update)
            .expect("broken merge update");
        assert_eq!(ob.best_ask_price, 1000);
        assert_eq!(ob.best_bid_price, 900);
    }

    #[test]
    fn already_received_update() {
        let snapshots = ParsedUpdate {
            last_update_id: 100000,
            bids: vec![Level {
                price: 8.0,
                amount: 1.0,
                exchange: "BITSTAMP".to_string(),
            }],
            asks: vec![Level {
                price: 10.0,
                amount: 1.0,
                exchange: "BITSTAMP".to_string(),
            }],
        };
        let mut ob = OrderBook::new(5, snapshots).unwrap();
        let new_update = ParsedUpdate {
            last_update_id: 9000,
            bids: vec![Level {
                price: 9.0,
                amount: 1.0,
                exchange: "BITSTAMP".to_string(),
            }],
            asks: vec![Level {
                price: 11.0,
                amount: 1.0,
                exchange: "BITSTAMP".to_string(),
            }],
        };
        ob.merge_parse_update(new_update)
            .expect("broken merge update");
        assert_eq!(ob.best_ask_price, 1000);
        assert_eq!(ob.best_bid_price, 800);
    }
}
