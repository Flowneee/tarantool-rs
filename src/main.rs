use futures::{SinkExt, TryStreamExt};
use tarantool_rs::{
    codec::request::{IProtoPing, IProtoRequest},
    connection::Connection,
};
use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let mut connection = Connection::new("127.0.0.1:3301").await.unwrap();
    connection
        .inner()
        .send(IProtoRequest::new(1, IProtoPing {}))
        .await
        .unwrap();
    let response = connection.inner().try_next().await.unwrap().unwrap();
    info!("Response: {:?}", response);
}
