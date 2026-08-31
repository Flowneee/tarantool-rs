use std::time::{Duration, Instant};

use futures::{StreamExt, stream::repeat_with};
use tarantool_rs::{Connection, ExecutorExt};

use tarantool_test_container::TarantoolTestContainer;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let container = TarantoolTestContainer::default();

    let conn = Connection::builder()
        .internal_simultaneous_requests_threshold(1000)
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
    loop {
        let _ = stream.next().await;
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
}
