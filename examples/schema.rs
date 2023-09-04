use tarantool_rs::{Connection, DmoOperation, Executor, ExecutorExt, IteratorType};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    pretty_env_logger::init();

    let conn = Connection::builder().build("127.0.0.1:3301").await?;
    let tx = conn.transaction().await?;

    let data = tx.space("clients").await?;
    info!("{:?}", data);
    let space = data.unwrap();
    info!(
        "Pre: {:?}",
        space
            .select::<(i64, String), _>(None, None, Some(IteratorType::All), ())
            .await?
    );
    info!(
        "UPSERT: {:?}",
        space
            .upsert(
                (0, "Name"),
                DmoOperation::string_splice("name", 2, 2, "!!".into()),
            )
            .await?
    );
    info!(
        "UPDATE: {:?}",
        space
            .update(
                (0,),
                (DmoOperation::string_splice("name", 2, 2, "!!".into()),)
            )
            .await?
    );
    info!("DELETE: {:?}", space.delete((2,)).await?);
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
