use anyhow::Result;
use tokio_stream::StreamExt;

use loshan_keyrock::orderbookaggregator::{
    orderbook_aggregator_client::OrderbookAggregatorClient, Empty,
};

// very simple client that prints the stream from the server to std output
// TODO print nicer
async fn book_summary_stream(
    mut client: OrderbookAggregatorClient<tonic::transport::Channel>,
) -> Result<()> {
    let empty_request = tonic::Request::new(Empty {});

    let mut stream = client.book_summary(empty_request).await?.into_inner();
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

// to be done very simple subscribe to server and print to std out
#[tokio::main]
async fn main() -> Result<()> {
    let client = OrderbookAggregatorClient::connect("http://127.0.0.1:5001").await?;

    _ = book_summary_stream(client).await?;

    Ok(())
}
