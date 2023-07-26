use crate::exchanges::ParsedUpdate;
use crate::orderbookaggregator::{Level, Summary};
use anyhow::Result;
use colored::Colorize;
use rust_decimal::{prelude::FromPrimitive, Decimal};
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fmt;

pub fn price_to_price_map_index(price: f64) -> usize {
    // representing with a very big usize was chosen over decimal for semplicity
    // and to remove bugs when prices were significantly below 1, like 0.00000001
    let price_index = (price * 1_000_000_000.0) as usize;
    return price_index;
}

pub fn volume_to_volume_mantissa(volume: f64) -> u32 {
    let price_index = Decimal::from_f64(volume).expect("Decimal failed to parse f64 for volume");
    return price_index.mantissa() as u32;
}

#[derive(Debug, Default)]
pub struct OrderBook {
    // The idea is storing price points in a BTreeMap
    // Indexing the price with some integer (usize) representation
    // This will allow O(1) retrieval for any price point
    // also the BTreeMap is ideal for sice of the price refence and keep tha data sorted
    pub best_bid_price: usize,
    pub best_ask_price: usize,
    bid_prices_reference: BTreeMap<usize, HashMap<String, Level>>,
    ask_prices_reference: BTreeMap<usize, HashMap<String, Level>>,
    pub reporting_levels: u32, // to be set as a parmater
    pub last_update_ids: HashMap<String, u64>,
}

// Summary trait to allow pretty printing from orderbook-client
impl fmt::Display for Summary {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Write strictly the first element into the supplied output
        // stream: `f`. Returns `fmt::Result` which indicates whether the
        // operation succeeded or failed. Note that `write!` uses syntax which
        // is very similar to `println!`.
        // let asks_to_display = format!("{:?}", self.asks);
        let mut asks_to_display = "".to_string();
        for level in self.asks.iter().rev() {
            let ask_level_to_print = format!(
                "{}: {} - {}\n",
                &level.exchange,
                level.price.to_string().red(),
                level.amount.to_string().red()
            );
            asks_to_display.push_str(&ask_level_to_print)
        }
        let mut bids_to_display = "".to_string();
        for level in self.bids.iter() {
            let bid_level_to_print = format!(
                "{}: {} - {}\n",
                &level.exchange,
                level.price.to_string().green(),
                level.amount.to_string().green()
            );
            bids_to_display.push_str(&bid_level_to_print)
        }

        write!(
            f,
            "{}: {}\n{}\n{}",
            "current spread",
            self.spread.to_string().green(),
            asks_to_display,
            bids_to_display
        )
    }
}

// levels - number of levels to be monitored as u32 (this is for the summary, the orderbook stores anyway everything it receives)
// ParsedUpdate - struct that cotains 2 vetors of levels (for bids and asks) and a timestamp
impl OrderBook {
    pub fn new(reporting_levels: u32, parsed_update: ParsedUpdate) -> Result<Self> {
        let best_bid_price: usize = 0;
        let best_ask_price: usize = usize::MAX;

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
        order_book.merge_parse_update(parsed_update)?;
        Ok(order_book)
    }

    pub fn merge_parse_update(&mut self, parsed_update: ParsedUpdate) -> Result<()> {
        // this first checks if for a given exchange we have a last_update_id timestmp
        // higher than current, if not simply returns Ok(())
        let exchange_identifier;
        if parsed_update.bids.len() >= 1 {
            exchange_identifier = parsed_update.bids[0].exchange.clone();
        } else {
            exchange_identifier = parsed_update.asks[0].exchange.clone();
        }
        if parsed_update.last_update_id
            > *self
                .last_update_ids
                .get(&exchange_identifier)
                .expect("failed to retrieve last_update_timestamp for exchange")
        {
            self.last_update_ids
                .insert(exchange_identifier, parsed_update.last_update_id);
        } else {
            return Ok(());
        }

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
        return Ok(());
    }

    pub fn merge_ask(&mut self, level: Level) -> Result<()> {
        let price_position = price_to_price_map_index(level.price);
        let ref_map = self
            .ask_prices_reference
            .entry(price_position)
            .or_insert(HashMap::new());

        if volume_to_volume_mantissa(level.amount) == 0 {
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
            let mut sorted_levels: Vec<(&std::string::String, &Level)> =
                exchange_levels_map.iter().collect();
            sorted_levels.sort_by(|a, b| b.1.amount.partial_cmp(&a.1.amount).unwrap());
            for (_, level) in sorted_levels {
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
            let mut sorted_levels: Vec<(&std::string::String, &Level)> =
                exchange_levels_map.iter().collect();
            sorted_levels.sort_by(|a, b| b.1.amount.partial_cmp(&a.1.amount).unwrap());
            for (_, level) in sorted_levels {
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
            spread: (self.best_ask_price as f64 / 1_000_000_000.0
                - self.best_bid_price as f64 / 1_000_000_000.0),
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
        assert_eq!(ob.best_ask_price, price_to_price_map_index(10.0));
        assert_eq!(ob.best_bid_price, price_to_price_map_index(8.0));
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
        assert_eq!(ob.best_bid_price, price_to_price_map_index(7.0));
        assert_eq!(ob.best_ask_price, price_to_price_map_index(11.0));
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
        assert_eq!(ob.best_ask_price, price_to_price_map_index(10.0));
        assert_eq!(ob.best_bid_price, price_to_price_map_index(9.0));
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
        assert_eq!(ob.best_ask_price, price_to_price_map_index(10.0));
        assert_eq!(ob.best_bid_price, price_to_price_map_index(8.0));
    }
}
