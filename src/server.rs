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

        _ = order_book.merge_parse_update(initial_bitstamp_snapshots);

        let output = async_stream::try_stream! {
            while let Some((key, message)) = stream_map.next().await {
                let message = message.expect("failed to unwrap message from streams main loop");
                
                tracing::info!("from exchange {} message received: {}", key, message);

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
                            tracing::info!(
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

                yield summary
            }
        };

        Ok(tonic::Response::new(
            Box::pin(output) as Self::BookSummaryStream
        ))
    }
}

// gRPC server main setup
#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber).expect("setting tracing default failed");

    let address = "127.0.0.1:5001";
    tracing::info!("Server up and running on {}", address);

    let socket_addr = address.parse()?;
    let orderbook_service = OrderbookAggregatorService::default();
    Server::builder()
        .add_service(OrderbookAggregatorServer::new(orderbook_service))
        .serve(socket_addr)
        .await?;
    Ok(())
}
