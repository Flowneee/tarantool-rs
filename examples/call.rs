#![allow(unused)]

use rmpv::Value;
use serde::Deserialize;
use tarantool_rs::{Connection, ExecutorExt, IteratorType};

#[derive(Debug, Deserialize)]
struct Response {
    first: u64,
    second: Option<Value>,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    pretty_env_logger::init();

    let conn = Connection::builder().build("127.0.0.1:3301").await?;

    let _ = conn
        .eval("function f(arg) return 42, nil end; return", ())
        .await?;

    // Drop call result
    let _ = conn.call("f", ()).await?;

    // Decode returned tuple entirely
    let resp: (u64, Value) = conn.call("f", ()).await?.decode_full()?;
    println!("{:?}", resp);

    // Decode first element
    let resp: u64 = conn.call("f", ()).await?.decode_first()?;
    println!("{:?}", resp);

    // Decode returned tuple as result. Since second element is null,
    // decode to Ok(u64)
    let resp: u64 = conn.call("f", ()).await?.decode_result()?;
    println!("{:?}", resp);

    // Decode returned tuple entirely into type
    let resp: Response = conn.call("f", ()).await?.decode_full()?;
    println!("{:?}", resp);

    Ok(())
}
