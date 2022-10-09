use futures::FutureExt;
use tarantool_rs::Connection;
use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let connection = Connection::builder()
        .auth("guest", None)
        .build("127.0.0.1:3301")
        .await
        .unwrap();
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
}
