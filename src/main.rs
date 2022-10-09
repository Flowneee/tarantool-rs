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

    tokio::try_join!(connection.ping(), connection.ping()).unwrap();
    connection.clone().ping().await.unwrap();

    let eval_response = connection
        .eval("return ...;", vec![10.into(), "qwe".into()])
        .await
        .unwrap();
    info!("Eval 'return ...;' response: {:?}", eval_response);
}
