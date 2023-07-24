use anyhow::Result;
use tokio_stream::StreamExt;

use loshan_keyrock::orderbookaggregator::{
    orderbook_aggregator_client::OrderbookAggregatorClient, SummaryRequest,
};

// very simple client that prints the stream from the server to std output
// TODO print nicer
async fn book_summary_stream(
    mut client: OrderbookAggregatorClient<tonic::transport::Channel>,
) -> Result<()> {
    let summary_request = SummaryRequest {
        levels: 10,
        symbol: "ethbtc".to_string(),
    };
    //let empty_request = tonic::Request::new(summary_request);

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

// to be done very simple subscribe to server and print to std out
#[tokio::main]
async fn main() -> Result<()> {
    // TODO: get order book reporting_levels and crypto pair from command line
    // pass it into book_summary_stream function and pass it along as part of the request
    let client = OrderbookAggregatorClient::connect("http://127.0.0.1:5001").await?;

    _ = book_summary_stream(client).await?;

    Ok(())
}
