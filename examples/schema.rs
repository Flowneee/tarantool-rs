use rmpv::Value;
use tarantool_rs::{Connection, ExecutorExt, IteratorType};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    pretty_env_logger::init();

    let conn = Connection::builder().build("127.0.0.1:3301").await?;

    let data = conn.space("clients").await?;
    info!("{:?}", data);
    let space = data.unwrap();
    info!(
        "Pre: {:?}",
        space
            .select::<(i64, String), _>(None, None, Some(IteratorType::All), ())
            .await?
    );
    space.upsert(("=",), (1, "Second")).await?;
    space
        .update(
            (0,),
            (Value::Array(vec![
                "=".into(),
                1.into(),
                "Second (updated)".into(),
            ]),),
        )
        .await?;
    info!(
        "Post: {:?}",
        space
            .index(1)
            .expect("Index with id 1 exists")
            .select::<(i64, String), _>(None, None, Some(IteratorType::All), ())
            .await?
    );
    Ok(())
}
