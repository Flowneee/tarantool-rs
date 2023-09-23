use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use futures::{future::try_join_all, TryFutureExt};
use tarantool_rs::{Connection, ExecutorExt};

type TarantoolTestContainer = tarantool_test_container::TarantoolTestContainer<
    tarantool_test_container::TarantoolDefaultArgs,
>;

pub fn compare_tarantool_rs_and_rusty_tarantool(c: &mut Criterion) {
    let mut group = c.benchmark_group("compare_with_rusty_tarantool");

    // Preparations
    let tokio_rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Tokio multithread runtime built");
    let container = TarantoolTestContainer::default();
    let tt_addr = || format!("127.0.0.1:{}", container.connect_port());
    let conn = tokio_rt
        .block_on(async { Connection::builder().build(tt_addr()).await })
        .expect("Connection created successfully");
    let conn_rusty = tokio_rt
        .block_on(async {
            let conn =
                rusty_tarantool::tarantool::ClientConfig::new(tt_addr(), "guest", "").build();
            conn.ping().await?;
            Result::<_, anyhow::Error>::Ok(conn)
        })
        .expect("Connection (RustyTarantool) created successfully");

    // Bench logic
    // NOTE: on my PC converting to join add slight overhead (1-2 microseconds for 1 future input)
    // NOTE: on my PC 50 input load tarantool to 50% on single core
    for parallel in [1, 2, 5, 10, 50].into_iter() {
        group.bench_with_input(
            BenchmarkId::new("tarantool_rs ping", parallel),
            &parallel,
            |b, p| {
                b.to_async(&tokio_rt).iter(|| async {
                    let make_fut = |_| conn.ping();
                    let futures = try_join_all((0..*p).map(make_fut));
                    futures.await.expect("Successful bench");
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new("tarantool_rs eval", parallel),
            &parallel,
            |b, p| {
                b.to_async(&tokio_rt).iter(|| async {
                    let make_fut = |_| {
                        conn.eval("return ...", (1, "two", true))
                            .and_then(|resp| async {
                                resp.decode_full::<(i64, String, bool)>()
                                    .map_err(Into::into)
                            })
                    };
                    let futures = try_join_all((0..*p).map(make_fut));
                    futures.await.expect("Successful bench");
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("RustyTarantool ping", parallel),
            &parallel,
            |b, p| {
                b.to_async(&tokio_rt).iter(|| async {
                    let make_fut = |_| conn_rusty.ping();
                    let futures = try_join_all((0..*p).map(make_fut));
                    futures.await.expect("Successful bench");
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new("RustyTarantool eval", parallel),
            &parallel,
            |b, p| {
                b.to_async(&tokio_rt).iter(|| async {
                    let make_fut = |_| {
                        conn_rusty
                            .eval("return ...", &(1, "two", true))
                            .and_then(|resp| async {
                                resp.decode::<(i64, String, bool)>().map_err(Into::into)
                            })
                    };
                    let futures = try_join_all((0..*p).map(make_fut));
                    futures.await.expect("Successful bench");
                })
            },
        );
    }

    group.finish();
}

criterion_group!(benches, compare_tarantool_rs_and_rusty_tarantool);
criterion_main!(benches);
