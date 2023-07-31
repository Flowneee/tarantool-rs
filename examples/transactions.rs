use tarantool_rs::{Connection, Executor, ExecutorExt, Value};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    pretty_env_logger::init();

    let connection = Connection::builder().build("127.0.0.1:3301").await?;
    connection.clone().ping().await?;

    let tx = connection.transaction().await?;
    let _: Value = tx.eval("box.space.clients:insert{2}", vec![]).await?;
    tx.rollback().await?;

    let tx = connection.transaction().await?;
    let _: Value = tx.eval("box.space.clients:insert{3}", vec![]).await?;
    drop(tx);

    let tx = connection.transaction().await?;
    let _: Value = tx.eval("box.space.clients:insert{4}", vec![]).await?;
    tx.commit().await?;

    let _: Vec<u32> = connection
        .select(
            512,
            0,
            None,
            None,
            Some(tarantool_rs::IteratorType::All),
            vec![1.into()],
        )
        .await?;
    Ok(())
}
