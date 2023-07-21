use loshan_keyrock::exchanges::{
    binance_diff_json_to_levels, bitstamp_json_to_levels, get_all_streams, get_binance_snapshot,
    get_bitstamp_snapshot,
}; // binance_json_to_levels this parses the book snapshots from stream not diffs, possibly useless
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
use async_stream::stream;
use futures::Stream;
use std::pin::Pin;
use tokio::{select, sync::mpsc};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tonic::{transport::Server, Request, Status};

#[derive(Debug, Default)]
struct OrderbookAggregatorService;

#[tonic::async_trait]
impl OrderbookAggregator for OrderbookAggregatorService {
    type BookSummaryStream = Pin<Box<dyn Stream<Item = Result<Summary, Status>> + Send>>;

    async fn book_summary(
        &self,
        _request: Request<Empty>,
    ) -> Result<tonic::Response<Self::BookSummaryStream>, Status> {
        let symbol = "btcusdt".to_string();

        // create streams before taking the 2 snapshots below
        let mut stream_map = get_all_streams(&symbol).await.unwrap();

        // get initial 2 snapshots here
        let initial_binance_snaphots = get_binance_snapshot(&symbol)
            .await
            .expect("Error getting ParsedUpdate for BINANCE snapshot");
        let initial_bitstamp_snapshots = get_bitstamp_snapshot(&symbol)
            .await
            .expect("Error getting ParsedUpdate for BITSTAMP snaphost");

        let mut order_book =
            OrderBook::new(10, initial_binance_snaphots).expect("failed to create new orderbook");
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

        let (sender, receiver) = mpsc::unbounded_channel();
        //tokio::spawn(async move {
        //loop {
        //select! {
        // start consuming from the streaming
        // Some((key, message)) = stream_map.next() => {
        while let Some((key, message)) = stream_map.next().await {
            let message = message.expect("failed to unwrap message from streams main loop");

            // bunch of printing for debussing purposes, TODO: remove

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
                    let subscription_event = &message_value["event"];

                    if subscription_event.as_str().unwrap() == "bts:subscription_succeeded" {
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
            // yield summary
            // if let Err(err) = sender.send(Ok(summary)) {
            //     //tracing::error!("Error sending summary: {:?}", err);
            //     return Err(Status::internal("Error sending summary"));
            // }
            //tx.send(Ok(feature.clone())).await.unwrap();
            _ = sender.send(Ok(summary))
        }
        // () = sender.closed() => {
        //     // tracing::info!("Client closed stream");
        //     // for (_, exchange_stream) in map.iter_mut() {
        //     //     exchange_stream.close(None).await.map_err(|_| Status::internal("Failed to close stream"))?;
        //     // }
        //     return Ok(());
        // },
        //}
        //}
        //});
        let output = UnboundedReceiverStream::new(receiver);
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
