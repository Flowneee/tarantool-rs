use tarantool_rs::{schema::SpaceMetadata, Connection, IteratorType};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    pretty_env_logger::init();

    let conn = Connection::builder().build("127.0.0.1:3301").await?;

    let data = conn.find_space_by_name("clients").await?;
    info!("{:?}", data);
    let space = data.unwrap();
    space.upsert(vec!["=".into()], vec![2.into()]).await?;
    info!(
        "{:?}",
        space
            .select::<(i64,)>(0, None, None, Some(IteratorType::All), vec![])
            .await?
    );
    Ok(())
}
