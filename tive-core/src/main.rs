use std::{
    net::{Ipv4Addr, SocketAddrV4},
    str::FromStr,
};

use warp::Filter;

#[tokio::main]
async fn main() {
    let hello = warp::path!("hello" / String).map(|name| format!("Hello, {}!", name));

    warp::serve(hello)
        .run(SocketAddrV4::new(
            Ipv4Addr::from_str("0.0.0.0").unwrap(),
            8080,
        ))
        .await;
}
