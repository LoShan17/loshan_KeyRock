use futures::{SinkExt, StreamExt, TryFutureExt}; //, TryFutureExt};
use futures::stream::SplitStream;
// use tokio::io::AsyncBufReadExt;
// use tokio::sync::mpsc;
use tokio::net::TcpStream;
use tokio_stream::StreamMap;
// use tokio_stream::StreamMap;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use reqwest;
use anyhow::{Context, Result};
// use tokio::io::AsyncWriteExt;
use serde_json; //::{Map, Value};

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

pub async fn get_binance_stream(symbol: &String) -> Result<SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>> {
    let ws_url_binance = url::Url::parse("wss://stream.binance.us:9443")
    .context("wrong binance url")?
    .join(&format!("/ws/{}@depth20@100ms", symbol))?;

    let (ws_stream_binance, _) = connect_async(&ws_url_binance)
    .await
    .context("Failed to connect to binance wss endpoint")?;

    let (_, read_stream) = ws_stream_binance.split();

    Ok(read_stream)
}


pub async fn get_bitstamp_stream(symbol: &String) -> Result<SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>> {
    let ws_url_bitstamp = url::Url::parse("wss://ws.bitstamp.net")
    .context("wrong bitstamp url")?;

    let (mut ws_stream_bitstamp, _) = connect_async(&ws_url_bitstamp)
    .await
    .context("Failed to connect to bitstamp wss endpoint")?;

    let subscribe_msg = serde_json::json!({
        "event": "bts:subscribe",
        "data": {
            "channel": format!("diff_order_book_{}", symbol)
        }
    });
    println!("{}", subscribe_msg);

    ws_stream_bitstamp.send(Message::Text(subscribe_msg.to_string())).await.unwrap();
    //ws_stream_bitstamp.next();

    println!("sent subscription message");
    let (_, read_stream) = ws_stream_bitstamp.split();
    // read_stream.next();

    Ok(read_stream)
}

// TODO: do this full implementation
pub async fn get_all_streams(symbol: String) -> Result<StreamMap<String, SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>>> {
    let mut streams_map = StreamMap::new();

    let binance_stream_read = get_binance_stream(&symbol).await.unwrap();
    streams_map.insert("BINANCE".to_string(), binance_stream_read);


    let bitstamp_stream_read = get_bitstamp_stream(&symbol).await.unwrap();
    streams_map.insert("BITSTAMP".to_string(), bitstamp_stream_read);

    println!("all streams returning");

    Ok(streams_map)
}


#[tokio::main]
async fn main() -> Result<()>{
    // careful with binance, apparently btcusd is not btcusd but the correct ticker is btcusdt
    let symbol = "btcusdt".to_string();
    
    // This works
    // let ws_read_stream = get_bitstamp_stream(&symbol).await.context("Error in getting bistamp stream").unwrap();

    // let ws_read_stream = get_binance_stream(&symbol).await.context("Error in getting bistamp stream").unwrap();
    let mut stream_map = get_all_streams(symbol).await.unwrap();
    while let Some((key, val)) = stream_map.next().await {
        println!("UPDATE RECEIVED");
        println!("{}", key);
        println!("{}", val.unwrap().into_text().unwrap());
    }

    // let read_future = stream_map.for_each(|message| async {
    //     println!("receiving...");
    //     let unwrapped_message = message.unwrap();
    //      //let data = unwrapped_message.into_data();
    //      let msg_str = unwrapped_message.into_text().unwrap();
    //      // tokio::io::stdout().write(&data).await.unwrap();
    //      println!("{}", msg_str);
    //      println!("received...");
    // });

    // read_future.await;

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