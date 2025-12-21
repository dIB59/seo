// benches/repository_benchmark.rs
use criterion::{criterion_group, criterion_main, Criterion};
use std::{hint::black_box, time::Duration};
use tokio::runtime::Runtime;

use app::{repository::sqlite::ResultsRepository, test_utils::set_up_test_db_with_prod_data};

fn bench_current_method(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let pool = rt.block_on(set_up_test_db_with_prod_data());
    let repo = ResultsRepository::new(pool);

    c.bench_function("get_result_by_job_id_12", |b| {
        b.to_async(&rt).iter(|| async {
            let result = repo
                .get_result_by_job_id(black_box(12))
                .await
                .expect("Failed");

            black_box(result)
        });
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(10)
        .measurement_time(Duration::from_secs(20));
    targets = bench_current_method
}

criterion_main!(benches);
