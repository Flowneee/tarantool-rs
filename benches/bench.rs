use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use futures::{future::try_join_all, TryFutureExt};
use tarantool_rs::{Connection, ExecutorExt};

type TarantoolTestContainer = tarantool_test_container::TarantoolTestContainer<
    tarantool_test_container::TarantoolDefaultArgs,
>;

pub fn bench_tarantool_rs(c: &mut Criterion) {
    let mut group = c.benchmark_group("tarantool_rs");

    // Preparations
    let tokio_rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Tokio multithread runtime built");
    let container = TarantoolTestContainer::default();
    let conn = tokio_rt
        .block_on(async {
            Connection::builder()
                .build(format!("127.0.0.1:{}", container.connect_port()))
                .await
        })
        .expect("Connection created successfully");

    // Bench logic
    // NOTE: on my PC converting to join add slight overhead (1-2 microseconds for 1 future input)
    // NOTE: on my PC 50 input load tarantool to 50% on single core
    for parallel in [1, 50, 250, 1000].into_iter() {
        group.bench_with_input(BenchmarkId::new("ping", parallel), &parallel, |b, p| {
            b.to_async(&tokio_rt).iter(|| async {
                let make_fut = |_| conn.ping();
                let futures = try_join_all((0..*p).map(make_fut));
                futures.await.expect("Successful bench");
            })
        });
        group.bench_with_input(BenchmarkId::new("call", parallel), &parallel, |b, p| {
            b.to_async(&tokio_rt).iter(|| async {
                let make_fut = |_| conn.call("gcinfo", ());
                let futures = try_join_all((0..*p).map(make_fut));
                futures.await.expect("Successful bench");
            })
        });
        group.bench_with_input(
            BenchmarkId::new("eval, pass and decode echo response", parallel),
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
    }

    group.finish();
}

criterion_group!(benches, bench_tarantool_rs,);
criterion_main!(benches);
