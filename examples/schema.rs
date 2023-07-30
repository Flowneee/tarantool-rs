use rmpv::Value;
use tarantool_rs::{Connection, IteratorType};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    pretty_env_logger::init();

    let conn = Connection::builder().build("127.0.0.1:3301").await?;

    let data = conn.find_space_by_name("clients").await?;
    info!("{:?}", data);
    let space = data.unwrap();
    info!(
        "Pre: {:?}",
        space
            .select::<(i64, String)>(0, None, None, Some(IteratorType::All), vec![])
            .await?
    );
    space
        .upsert(vec!["=".into()], vec![2.into(), "Second".into()])
        .await?;
    space
        .update(
            0,
            vec![2.into()],
            vec![Value::Array(vec![
                "=".into(),
                2.into(),
                "Second (updated)".into(),
            ])],
        )
        .await?;
    info!(
        "Post: {:?}",
        space
            .select::<(i64, String)>(0, None, None, Some(IteratorType::All), vec![])
            .await?
    );
    Ok(())
}
