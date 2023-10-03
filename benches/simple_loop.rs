use std::time::{Duration, Instant};

use futures::{stream::repeat_with, StreamExt};
use tarantool_rs::{Connection, ExecutorExt};

type TarantoolTestContainer = tarantool_test_container::TarantoolTestContainer<
    tarantool_test_container::TarantoolDefaultArgs,
>;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let container = TarantoolTestContainer::default();

    let conn = Connection::builder()
        .dispatcher_internal_queue_size(1100)
        .build(format!("127.0.0.1:{}", container.connect_port()))
        .await?;
    // let conn = rusty_tarantool::tarantool::ClientConfig::new(
    //     format!("127.0.0.1:{}", container.connect_port()),
    //     "guest",
    //     "",
    // )
    // .build();
    // conn.ping().await?;

    let mut counter = 0u64;
    let mut last_measured_counter = 0;
    let mut last_measured_ts = Instant::now();

    let interval_secs = 2;
    let interval = Duration::from_secs(interval_secs);

    let mut stream = repeat_with(|| conn.ping()).buffer_unordered(1000);
    while let _ = stream.next().await {
        counter += 1;
        if last_measured_ts.elapsed() > interval {
            last_measured_ts = Instant::now();
            let counter_diff = counter - last_measured_counter;
            last_measured_counter = counter;
            println!(
                "Iterations over last {interval_secs} seconds: {counter_diff}, per second: {}",
                counter_diff / interval_secs
            );
        }
    }

    Ok(())
}
