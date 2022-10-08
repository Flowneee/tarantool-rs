use tarantool_rs::Connection;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let connection = Connection::builder().build("127.0.0.1:3301").await.unwrap();
    tokio::try_join!(connection.ping(), connection.ping()).unwrap();
    connection.clone().ping().await.unwrap();
}
