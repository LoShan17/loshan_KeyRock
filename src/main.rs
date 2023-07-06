use futures::{SinkExt, StreamExt, TryFutureExt};
use tokio::io::AsyncBufReadExt;
use tokio::sync::mpsc;
use tokio::net::TcpStream;
// use tokio_stream::StreamMap;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use reqwest;
use anyhow::{Context, Result};

// OK
pub async fn get_bitstamp_snapshot(symbol: &String) -> Result<String> {
    let url = format!(
        "https://www.bitstamp.net/api/v2/order_book/{}/",
        symbol.to_lowercase()
    );

    let snapshot = reqwest::get(url).await?;
    let body = snapshot.text().await?;
    Ok(body)
}

// OK
pub async fn get_binance_snapshot(symbol: &String) -> Result<String> {
    let url = format!(
        "https://www.binance.us/api/v3/depth?symbol={}&limit=1000",
        symbol.to_uppercase()
    );

    let snapshot = reqwest::get(url).await?;
    let body = snapshot.text().await?;
    Ok(body)
}

pub async fn get_binance_stream(symbol: &String) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>> {
    let ws_url_binance = url::Url::parse("wss://stream.binance.us:9443")
    .context("bad binance url")?
    .join(&format!("ws/{}@depth@100ms", symbol))?;

    let (ws_stream_binance, _) = connect_async(&ws_url_binance)
    .await
    .context("Failed to connect to binance wss endpoint")?;
    Ok(ws_stream_binance)
}


#[tokio::main]
async fn main() -> Result<()>{
    let connect_addr = "wss://ws.bitstamp.net";
    let bitstamp_url = url::Url::parse(&connect_addr).context("Error parsing URL")?; // remember that ? is dependent on complete awaitable block/function, with correct return type and signature
    let (mut ws_stream, _) = connect_async(&bitstamp_url).await.expect("Failed to connect");
    Ok(())
}




// // Working single queries snapshots
// #[tokio::main]
// async fn main() {
//     let symbol = "ethbtc".to_string();
//     let bitsamp_string_snapshot = get_bitstamp_snapshot(&symbol).await;

//     let binance_string_snapshot = get_binance_snapshot(&symbol).await;
    
//     println!("{}", &bitsamp_string_snapshot.expect("bitsamp snapshot returned error")[..10000]);
//     println!("{}", "JUST printed bitstamp".to_string());
//     println!("{}", binance_string_snapshot.expect("binance snapshot returned error"));
//     println!("{}", "JUST printed binance".to_string());
// }