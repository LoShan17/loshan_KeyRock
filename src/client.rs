use anyhow::Result;
use clap::Parser;
use clearscreen;
use tokio_stream::StreamExt;

use loshan_keyrock::orderbookaggregator::{
    orderbook_aggregator_client::OrderbookAggregatorClient, SummaryRequest,
};

#[derive(Parser)]
struct Cli {
    symbol: String,
    levels: u32,
}

async fn book_summary_stream(
    mut client: OrderbookAggregatorClient<tonic::transport::Channel>,
    symbol: String,
    levels: u32,
) -> Result<()> {
    let summary_request = SummaryRequest { levels, symbol };

    let mut stream = client.book_summary(summary_request).await?.into_inner();
    while let Some(summary) = stream.next().await {
        clearscreen::clear().expect("failed to clear screen");
        match summary {
            Ok(summary) => println!("{}", summary),
            Err(err) => {
                return Err(err.into());
            }
        };
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let client = OrderbookAggregatorClient::connect("http://127.0.0.1:5001").await?;

    let args = Cli::parse();
    _ = book_summary_stream(client, args.symbol, args.levels).await?;

    Ok(())
}
