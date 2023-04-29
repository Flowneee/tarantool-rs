use futures::FutureExt;
use tarantool_rs::{Connection, ConnectionLike, Value};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt::init();

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

    // let eval_fut = connection
    //     .eval::<_, (u32, String)>(
    //         "fiber = require('fiber'); fiber.sleep(0.5); return ...;",
    //         vec![42.into(), "pong".into()],
    //     )
    //     .inspect(|res| info!("Eval response: {:?}", res));
    // let call_fut = connection
    //     .call::<_, (String,)>("tostring", vec![42.into()])
    //     .inspect(|res| info!("Call response: {:?}", res));
    // let ping_fut = connection
    //     .ping()
    //     .inspect(|res| info!("Ping response: {:?}", res));
    // let _ = tokio::join!(eval_fut, call_fut, ping_fut);

    // let stream = connection.stream();
    // let eval_fut = stream
    //     .eval::<_, (u32, String)>(
    //         "fiber = require('fiber'); fiber.sleep(0.5); return ...;",
    //         vec![42.into(), "pong".into()],
    //     )
    //     .inspect(|res| info!("Eval response: {:?}", res));
    // let call_fut = stream
    //     .call::<_, (String,)>("tostring", vec![42.into()])
    //     .inspect(|res| info!("Call response: {:?}", res));
    // let ping_fut = stream
    //     .ping()
    //     .inspect(|res| info!("Ping response: {:?}", res));
    // let _ = tokio::join!(eval_fut, call_fut, ping_fut);

    // let transaction = connection.transaction().await.unwrap();
    // let eval_fut = transaction
    //     .eval::<_, (u32, String)>(
    //         "fiber = require('fiber'); fiber.sleep(0.5); return ...;",
    //         vec![42.into(), "pong".into()],
    //     )
    //     .inspect(|res| info!("Eval response: {:?}", res));
    // let call_fut = transaction
    //     .call::<_, (String,)>("tostring", vec![42.into()])
    //     .inspect(|res| info!("Call response: {:?}", res));
    // let ping_fut = transaction
    //     .ping()
    //     .inspect(|res| info!("Ping response: {:?}", res));
    // let _ = tokio::join!(eval_fut, call_fut, ping_fut);
    Ok(())
}
