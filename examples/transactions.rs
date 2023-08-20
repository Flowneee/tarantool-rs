use tarantool_rs::{Connection, Executor, ExecutorExt};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    pretty_env_logger::init();

    let connection = Connection::builder().build("127.0.0.1:3301").await?;
    connection.clone().ping().await?;

    let tx = connection.transaction().await?;
    let _ = tx.eval("box.space.clients:insert{2}", ()).await?;
    tx.rollback().await?;

    let tx = connection.transaction().await?;
    let _ = tx.eval("box.space.clients:insert{3}", ()).await?;
    drop(tx);

    let tx = connection.transaction().await?;
    let _ = tx.eval("box.space.clients:insert{4}", ()).await?;
    tx.commit().await?;

    let _: Vec<u32> = connection
        .select(
            512,
            0,
            None,
            None,
            Some(tarantool_rs::IteratorType::All),
            (1,),
        )
        .await?;
    Ok(())
}
