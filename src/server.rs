use loshan_keyrock::exchanges::{
    binance_json_to_levels, bitstamp_json_to_levels, get_all_streams, get_binance_snapshot,
    get_bitstamp_snapshot,
};
use loshan_keyrock::orderbook::OrderBook;
use loshan_keyrock::orderbookaggregator::{
    orderbook_aggregator_server::{OrderbookAggregator, OrderbookAggregatorServer},
    Empty, Summary,
};
// use loshan_keyrock::orderbookaggregator::{
//     orderbook_aggregator_client::OrderbookAggregatorClient, Empty,
// };

use anyhow::{Context, Result};
use futures::StreamExt; //, TryFutureExt}; {SinkExt,
use serde_json;
// use crate::serde_json::Error;
// use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use futures::Stream;
use std::pin::Pin;
use tonic::{Request, Response, Status};

#[derive(Debug)]
struct OrderbookAggregatorService;

#[tonic::async_trait]
impl OrderbookAggregator for OrderbookAggregatorService {
    type BookSummaryStream = Pin<Box<dyn Stream<Item = Result<Summary, Status>> + Send>>;

    async fn book_summary(
        &self,
        _request: Request<Empty>,
    ) -> Result<tonic::Response<Self::BookSummaryStream>, Status> {
        // let mut ob = OrderBook::new(symbol, reporting_levels, parsed_update);

        // Ok(tonic::Response::new(
        //     Box::pin(stream) as Self::BookSummaryStream
        // ))
        unimplemented!();
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // careful with binance, apparently btcusd is not provided on stream and there is only btcusdt
    let symbol = "btcusdt".to_string();

    // get initial 2 snapshots here
    // create orderbook and start stream
    let initial_binance_snaphots = get_binance_snapshot(&symbol)
        .await
        .expect("Error getting ParsedUpdate for BINANCE snapshot");
    let initial_bitstamp_snapshots = get_bitstamp_snapshot(&symbol)
        .await
        .expect("Error getting ParsedUpdate for BITSTAMP snaphost");

    let mut order_book =
        OrderBook::new(10, initial_binance_snaphots).expect("failed to create new orderbook");
    _ = order_book.merge_parse_update(initial_bitstamp_snapshots);

    let mut stream_map = get_all_streams(&symbol).await.unwrap();
    while let Some((key, message)) = stream_map.next().await {
        let message = message.map_err(|_| Status::internal("Failed to get message"))?;

        let message = match message {
            tungstenite::Message::Text(_) => message,
            // trying to just skip Pings and Pongs messages otherwise they will break parsing
            tungstenite::Message::Ping(_) => {
                continue;
            }
            tungstenite::Message::Pong(_) => {
                continue;
            }
            _ => {
                panic!("unknown message received from stream")
            }
        };

        let message_value: serde_json::Value =
            serde_json::from_slice(&message.into_data()).expect("empty message?");

        let parsed_update = match key {
            "BINANCE" => binance_json_to_levels(message_value)
                .expect("error in binance json value to updates"),
            "BITSTAMP" => {
                let subscription_event = &message_value["event"];

                // replace the below with match and "data" in the second branch
                if subscription_event
                    .as_str()
                    .context("can't parse event field")?
                    == "bts:subscription_succeeded"
                {
                    println!("received subscription confirmation message with no data, continue");
                    continue;
                } else {
                    bitstamp_json_to_levels(&message_value)
                        .expect("error in bitstamp json value to updates")
                }
            }
            _ => panic!("not implemented exchange"),
        };
        _ = order_book.merge_parse_update(parsed_update);

        let summary = order_book.get_summary().expect("broke in creating summary");

        println!("PRINTING SUMMARY");
        println!("{:?}", summary);
        println!("length of bids {}", summary.bids.len());
        println!("length of asks {}", summary.asks.len());
        println!("END SUMMARY");
    }

    Ok(())
}
