use anyhow::Result;
use tokio_stream::StreamExt;

use loshan_keyrock::orderbookaggregator::{
    orderbook_aggregator_client::OrderbookAggregatorClient, Empty,
};

// to be done very simple subscribe to server and print to std out
#[tokio::main]
async fn main() -> Result<()> {
    Ok(())
}
