use anyhow::Result;
use clap::Parser;
use tokio_stream::StreamExt;

use loshan_keyrock::orderbookaggregator::{
    orderbook_aggregator_client::OrderbookAggregatorClient, SummaryRequest,
};

#[derive(Parser)]
struct Cli {
    symbol: String,
    levels: u32,
}

// TODO print nicer
async fn book_summary_stream(
    mut client: OrderbookAggregatorClient<tonic::transport::Channel>,
    symbol: String,
    levels: u32,
) -> Result<()> {
    let summary_request = SummaryRequest { levels, symbol };

    let mut stream = client.book_summary(summary_request).await?.into_inner();
    while let Some(summary) = stream.next().await {
        match summary {
            Ok(summary) => println!("\n{:#?}", summary),
            Err(err) => {
                return Err(err.into());
            }
        };
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // TODO: get order book reporting_levels and crypto pair from command line
    // pass it into book_summary_stream function and pass it along as part of the request
    let client = OrderbookAggregatorClient::connect("http://127.0.0.1:5001").await?;

    let args = Cli::parse();
    _ = book_summary_stream(client, args.symbol, args.levels).await?;

    Ok(())
}
