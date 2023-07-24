use anyhow::Result;
use async_stream;
use futures::Stream;
use futures::StreamExt;
use loshan_keyrock::exchanges::{
    binance_diff_json_to_levels, bitstamp_json_to_levels, get_all_streams, get_binance_snapshot,
    get_bitstamp_snapshot,
};
use loshan_keyrock::orderbook::OrderBook;
use loshan_keyrock::orderbookaggregator::{
    orderbook_aggregator_server::{OrderbookAggregator, OrderbookAggregatorServer},
    Summary, SummaryRequest,
};
use serde_json;
use std::pin::Pin;
use tonic::{transport::Server, Request, Status};

#[derive(Debug, Default)]
struct OrderbookAggregatorService;

#[tonic::async_trait]
impl OrderbookAggregator for OrderbookAggregatorService {
    type BookSummaryStream = Pin<Box<dyn Stream<Item = Result<Summary, Status>> + Send>>;

    async fn book_summary(
        &self,
        request: Request<SummaryRequest>,
    ) -> Result<tonic::Response<Self::BookSummaryStream>, Status> {
        // TODO, get both symbol and orderbook reporting_levels from SummaryRequest
        // let symbol = "ethbtc".to_string();
        let SummaryRequest { symbol, levels } = request.into_inner();

        // create streams before taking the 2 snapshots below
        let mut stream_map = get_all_streams(&symbol)
            .await
            .expect("Error in getting exchenges Streams Map");

        // get initial 2 snapshots here
        let initial_binance_snaphots = get_binance_snapshot(&symbol)
            .await
            .expect("Error getting ParsedUpdate for BINANCE snapshot");
        let initial_bitstamp_snapshots = get_bitstamp_snapshot(&symbol)
            .await
            .expect("Error getting ParsedUpdate for BITSTAMP snaphost");

        let mut order_book = OrderBook::new(levels, initial_binance_snaphots)
            .expect("failed to create new orderbook");
        println!("original binance snapshot print");
        println!(
            "bb: {}, ba: {}",
            order_book.best_bid_price, order_book.best_ask_price
        );
        _ = order_book.merge_parse_update(initial_bitstamp_snapshots);
        println!(
            "bb: {}, ba: {}",
            order_book.best_bid_price, order_book.best_ask_price
        );

        let output = async_stream::try_stream! {
            while let Some((key, message)) = stream_map.next().await {
                let message = message.expect("failed to unwrap message from streams main loop");

                // bunch of printing for debugging purposes, TODO: remove

                println!("{}", key);
                println!("this was the message: {}", message);

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
                    "BINANCE" => binance_diff_json_to_levels(message_value)
                        .expect("error in binance json value to updates"),
                    "BITSTAMP" => {
                        let bitstamp_event = &message_value["event"];

                        if bitstamp_event.as_str().expect("error in parsing bitstamp event to string") == "bts:subscription_succeeded" {
                            println!(
                                "received subscription confirmation message with no data, continue"
                            );
                            continue;
                        } else {
                            bitstamp_json_to_levels(&message_value)
                                .expect("error in bitstamp json value to updates")
                        }
                    }
                    _ => panic!("not implemented exchange"),
                };
                _ = order_book.merge_parse_update(parsed_update);

                let summary = order_book.get_summary().expect("Error in creating summary");

                // bunch of printing for debussing purposes, TODO: remove
                println!("PRINTING SUMMARY");
                println!("{:?}", summary);
                println!("length of bids {}", summary.bids.len());
                println!("length of asks {}", summary.asks.len());
                println!("END SUMMARY");
                yield summary
            }
        };

        Ok(tonic::Response::new(
            Box::pin(output) as Self::BookSummaryStream
        ))
    }
}

// attempt gRPC server main setup
#[tokio::main]
async fn main() -> Result<()> {
    let addr = "127.0.0.1:5001";
    println!("Server up and running on {}", addr);

    let socket_addr = addr.parse()?;
    let orderbook = OrderbookAggregatorService::default();
    Server::builder()
        .add_service(OrderbookAggregatorServer::new(orderbook))
        .serve(socket_addr)
        .await?;
    Ok(())
}
