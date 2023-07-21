KeyRock Challenge defined as per Rust L2.pdf

cargo run --bin orderbook-server

and after the server is up and running

cargo run --bin orderbook-client, to see summaries (as defined in orderbookaggregator.proto) printed to standar output

References use for several topics included below:

Rust General:
https://doc.rust-lang.org/cargo/guide/project-layout.html
https://stackoverflow.com/questions/57756927/rust-modules-confusion-when-there-is-main-rs-and-lib-rs

OrderBook:
https://sanket.tech/posts/rustbook/
https://github.com/inv2004/orderbook-rs/blob/master/src/ob.rs
https://stackoverflow.com/questions/30851464/https://doc.rust-lang.org/std/collections/hash_map/enum.Entry.html

Streams:
https://github.com/snapview/tokio-tungstenite/issues/137
https://docs.rs/tokio-stream/latest/tokio_stream/struct.StreamMap.html
https://docs.rs/futures/latest/futures/stream/struct.SplitStream.html

https://github.com/binance/binance-spot-api-docs/blob/master/web-socket-streams.md
notes explaining how to take an initial snaphot of exchnages and apllying the diff is actually the right thing to do for Binance, and since we are doing it also for Bitstamp

Grcp server/client streaming example
https://github.com/hyperium/tonic/blob/master/examples/routeguide-tutorial.md

Tests
https://doc.rust-lang.org/rust-by-example/testing/unit_testing.html#:~:text=The%20bodies%20of%20test%20functions,in%20the%20test%20function%20panics.