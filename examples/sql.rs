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

    info!(
        "INSERT row count {}",
        conn.execute_sql(
            "INSERT INTO \"clients\" (\"id\", \"name\") VALUES (?, ?), (?, ?)",
            (99, "SQL", 100, "SQL"),
        )
        .await?
        .row_count()?
    );

    info!(
        "DELETE row count {}",
        conn.execute_sql("DELETE FROM \"clients\" WHERE \"name\" = ?", ("SQL",))
            .await?
            .row_count()?
    );

    Ok(())
}
