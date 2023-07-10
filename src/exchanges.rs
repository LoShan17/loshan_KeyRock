use futures::{SinkExt, StreamExt}; //, TryFutureExt};
use futures::stream::SplitStream;
use prost::encoding::message;
use tonic::{transport::Server, Status};
use tokio::net::TcpStream;
use tokio_stream::StreamMap;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use reqwest;
use anyhow::{Context, Result};
use serde_json;

// const EXCHANGES:Vec<String> = vec!["BINANCE".to_string(), "BITSTAMP".to_string()];

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
pub async fn get_all_streams(symbol: String) -> Result<StreamMap<&'static str, SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>>> {
    let mut streams_map = StreamMap::new();

    // for exchange in EXCHANGES::iter() {
    //     match exchange {
    //         "BINANCE" => {}
            
    //     }

    // }

    let binance_stream_read = get_binance_stream(&symbol).await.unwrap();
    streams_map.insert("BINANCE", binance_stream_read);


    let bitstamp_stream_read = get_bitstamp_stream(&symbol).await.unwrap();
    streams_map.insert("BITSTAMP", bitstamp_stream_read);

    println!("all streams returning");

    Ok(streams_map)
}