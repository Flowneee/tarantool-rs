use futures::FutureExt;
use tarantool_rs::{Connection, ConnectionLike};
use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let connection = Connection::builder().build("127.0.0.1:3301").await.unwrap();
    connection.clone().ping().await.unwrap();

    let eval_fut = connection
        .eval(
            "fiber = require('fiber'); fiber.sleep(0.5); return ...;",
            vec![42.into(), "pong".into()],
        )
        .inspect(|res| info!("Eval response: {:?}", res));
    let call_fut = connection
        .call("tostring", vec![42.into()])
        .inspect(|res| info!("Call response: {:?}", res));
    let ping_fut = connection
        .ping()
        .inspect(|res| info!("Ping response: {:?}", res));
    let _ = tokio::join!(eval_fut, call_fut, ping_fut);

    let stream = connection.stream();
    let eval_fut = stream
        .eval(
            "fiber = require('fiber'); fiber.sleep(0.5); return ...;",
            vec![42.into(), "pong".into()],
        )
        .inspect(|res| info!("Eval response: {:?}", res));
    let call_fut = stream
        .call("tostring", vec![42.into()])
        .inspect(|res| info!("Call response: {:?}", res));
    let ping_fut = stream
        .ping()
        .inspect(|res| info!("Ping response: {:?}", res));
    let _ = tokio::join!(eval_fut, call_fut, ping_fut);

    let transaction = connection.transaction().await.unwrap();
    let eval_fut = transaction
        .eval(
            "fiber = require('fiber'); fiber.sleep(0.5); return ...;",
            vec![42.into(), "pong".into()],
        )
        .inspect(|res| info!("Eval response: {:?}", res));
    let call_fut = transaction
        .call("tostring", vec![42.into()])
        .inspect(|res| info!("Call response: {:?}", res));
    let ping_fut = transaction
        .ping()
        .inspect(|res| info!("Ping response: {:?}", res));
    let _ = tokio::join!(eval_fut, call_fut, ping_fut);
}
