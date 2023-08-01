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

    let _: Value = conn
        .eval("function f(arg) return 42, nil end; return", vec![])
        .await?;

    let resp: Value = conn.call("f", vec![]).await?;
    println!("{:?}", resp);

    let resp: (Value, Value) = conn.call("f", vec![]).await?;
    println!("{:?}", resp);

    let resp: Vec<Value> = conn.call("f", vec![]).await?;
    println!("{:?}", resp);

    let resp: (u64, Option<String>) = conn.call("f", vec![]).await?;
    println!("{:?}", resp);

    let resp: Response = conn.call("f", vec![]).await?;
    println!("{:?}", resp);

    Ok(())
}
