use tarantool_rs::{schema::SpaceMetadata, Connection};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    pretty_env_logger::init();

    let conn = Connection::builder().build("127.0.0.1:3301").await?;

    let data = SpaceMetadata::load_by_name(conn, "clients").await?;
    info!("{:?}", data);

    Ok(())
}
