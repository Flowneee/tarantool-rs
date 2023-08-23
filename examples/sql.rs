use tarantool_rs::{Connection, ExecutorExt};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    pretty_env_logger::init();

    let conn = Connection::builder().build("127.0.0.1:3301").await?;

    info!(
        "{:?}",
        conn.execute_sql("SELECT * FROM \"clients\"", ())
            .await?
            .decode_select::<(u64, String)>()?
    );

    let prepared_insert = dbg!(
        conn.prepare_sql("INSERT INTO \"clients\" (\"id\", \"name\") VALUES (?, ?), (?, ?)")
            .await?
    );
    info!(
        "INSERT row count {}",
        prepared_insert
            .execute((99, "SQL", 100, "SQL"),)
            .await?
            .row_count()?
    );

    info!(
        "DELETE row count {}",
        conn.execute_sql("DELETE FROM \"clients\" WHERE \"name\" = ?", ("SQL",))
            .await?
            .row_count()?
    );

    info!(
        "{:?}",
        conn.execute_sql("SELECT * FROM \"clients\"", ())
            .await?
            .decode_select::<(u64, String)>()?
    );

    Ok(())
}
