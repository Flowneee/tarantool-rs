use tarantool_rs::{schema::SpaceMetadata, Connection};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt::init();

    let conn = Connection::builder().build("127.0.0.1:3301").await?;

    //let data = get_list_of_user_spaces(&conn).await?;
    let data = SpaceMetadata::load_by_name(conn, "clients").await?;
    info!("{:?}", data);

    Ok(())
}
