use crate::orderbookaggregator::Level;
use anyhow::{Context, Result};
use futures::stream::SplitStream;
use futures::{SinkExt, StreamExt}; //, TryFutureExt};
use prost::encoding::message;
use reqwest;
use serde_json::Value;
use tokio::net::TcpStream;
use tokio_stream::StreamMap;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use tonic::{transport::Server, Status};

// const EXCHANGES:Vec<String> = vec!["BINANCE".to_string(), "BITSTAMP".to_string()];
#[derive(Debug, Default)]
pub struct ParsedUpdate {
    bids: Vec<Level>,
    asks: Vec<Level>,
    last_update_id: u64,
}
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

pub async fn get_binance_stream(
    symbol: &String,
) -> Result<SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>> {
    let ws_url_binance = url::Url::parse("wss://stream.binance.us:9443")
        .context("wrong binance url")?
        .join(&format!("/ws/{}@depth10@100ms", symbol))?;

    let (ws_stream_binance, _) = connect_async(&ws_url_binance)
        .await
        .context("Failed to connect to binance wss endpoint")?;

    let (_, read_stream) = ws_stream_binance.split();

    Ok(read_stream)
}

pub async fn get_bitstamp_stream(
    symbol: &String,
) -> Result<SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>> {
    let ws_url_bitstamp = url::Url::parse("wss://ws.bitstamp.net").context("wrong bitstamp url")?;

    let (mut ws_stream_bitstamp, _) = connect_async(&ws_url_bitstamp)
        .await
        .context("Failed to connect to bitstamp wss endpoint")?;

    // try these 3?
    // diff_order_book_[currency_pair]
    // detail_order_book_[currency_pair]
    // order_book_[currency_pair]

    // from binance https://github.com/binance/binance-spot-api-docs/blob/master/web-socket-streams.md
    // it seems that taking a snapshot and applying the diff feed is the only way.
    // maybe worth keeping it consistent across the 2 exchanges and do it in a similar way

    let subscribe_msg = serde_json::json!({
        "event": "bts:subscribe",
        "data": {
            "channel": format!("order_book_{}", symbol)
        }
    });
    println!("{}", subscribe_msg);

    ws_stream_bitstamp
        .send(Message::Text(subscribe_msg.to_string()))
        .await
        .context("failed to subscribe to bitstap")?;

    println!("sent subscription message");
    let (_, read_stream) = ws_stream_bitstamp.split();

    // try receiving the first message (normally subscription confirmation before adding the stream to the map)
    // let _ = read_stream.next();

    Ok(read_stream)
}

// TODO: do this full implementation
pub async fn get_all_streams(
    symbol: String,
) -> Result<StreamMap<&'static str, SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>>> {
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

// for now the idea is simply to return a PrasedUpdate type/ maybe worth switching to HashMap instead of a Vec
// with lareday as key the u64 from the mantissa of the float price?
// with Levels as defined under .proto

pub fn bitstamp_json_to_levels(value: &Value) -> Result<ParsedUpdate> {
    let mut vector_of_bids: Vec<Level> = Vec::with_capacity(
        value["data"]["bids"]
            .as_array()
            .expect("failed to get bids capacity")
            .len(),
    );
    let mut vector_of_asks: Vec<Level> = Vec::with_capacity(
        value["data"]["asks"]
            .as_array()
            .expect("failed to get asks capacity")
            .len(),
    );
    let last_update_id = value["data"]["microtimestamp"]
        .as_str()
        .context("failed to parse microtimestamp as string")?
        .parse::<u64>()
        .context(" failed to parse from string to u64")?;

    for bid in value["data"]["bids"]
        .as_array()
        .context("no array for bids in bitstamp message")?
    {
        let level = Level {
            price: bid[0]
                .as_str()
                .context("bitstamp bid price failed as string")?
                .parse::<f64>()
                .context("bitstamp bid price failed as float")?,
            amount: bid[1]
                .as_str()
                .context("bitstamp bid amount failed as string")?
                .parse::<f64>()
                .context("bitstamp bid amount failed as float")?,
            exchange: "BITSTAMP".to_string(),
        };
        vector_of_bids.insert(0, level);
    }

    for ask in value["data"]["asks"]
        .as_array()
        .context("no array for asks in bitsamp message")?
    {
        let level = Level {
            price: ask[0]
                .as_str()
                .context("bitstamp ask price failed as string")?
                .parse::<f64>()
                .context("bitstamp ask price failed as float")?,
            amount: ask[1]
                .as_str()
                .context("bitstampask amount failed as string")?
                .parse::<f64>()
                .context("bitstamp ask amount failed as float")?,
            exchange: "BINANCE".to_string(),
        };
        vector_of_asks.insert(0, level);
    }

    Ok(ParsedUpdate {
        bids: vector_of_bids,
        asks: vector_of_asks,
        last_update_id,
    })
}

pub fn binance_json_to_levels(value: Value) -> Result<ParsedUpdate> {
    let mut vector_of_bids: Vec<Level> =
        Vec::with_capacity(value["bids"].as_array().unwrap().len());
    let mut vector_of_asks: Vec<Level> =
        Vec::with_capacity(value["asks"].as_array().unwrap().len());
    let last_update_id = value["lastUpdateId"].as_u64().unwrap();

    for bid in value["bids"]
        .as_array()
        .context("no array for bids in binance message")?
    {
        // for number in bid.as_array().context("")? {

        // }
        let level = Level {
            price: bid[0]
                .as_str()
                .context("binance bid price failed as string")?
                .parse::<f64>()
                .context("binance bid price failed as float")?,
            amount: bid[1]
                .as_str()
                .context("binance bid amount failed as string")?
                .parse::<f64>()
                .context("binance bid amount failed as float")?,
            exchange: "BINANCE".to_string(),
        };
        vector_of_bids.insert(0, level);
    }

    for ask in value["asks"]
        .as_array()
        .context("no array for asks in binance message")?
    {
        let level = Level {
            price: ask[0]
                .as_str()
                .context("binance ask price failed as string")?
                .parse::<f64>()
                .context("binance ask price failed as float")?,
            amount: ask[1]
                .as_str()
                .context("binance ask amount failed as string")?
                .parse::<f64>()
                .context("binance ask amount failed as float")?,
            exchange: "BINANCE".to_string(),
        };
        vector_of_asks.insert(0, level);
    }

    Ok(ParsedUpdate {
        bids: vector_of_bids,
        asks: vector_of_asks,
        last_update_id,
    })
}
