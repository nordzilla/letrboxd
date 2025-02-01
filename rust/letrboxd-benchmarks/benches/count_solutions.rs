use criterion::{black_box, criterion_group, criterion_main, Criterion};
use letrboxd_benchmarks::{
  count_solutions, solve_filter_only, solve_partition, solve_partition_once, TEST_INPUT,
};

fn bench_count_solutions(c: &mut Criterion) {
  let mut group = c.benchmark_group("LetrBoxd Count Solutions");

  group.bench_function("filter_only", |b| {
    b.iter(|| count_solutions(black_box(TEST_INPUT), black_box(solve_filter_only)));
  });
  group.bench_function("partition", |b| {
    b.iter(|| count_solutions(black_box(TEST_INPUT), black_box(solve_partition)));
  });
  group.bench_function("partition_once", |b| {
    b.iter(|| count_solutions(black_box(TEST_INPUT), black_box(solve_partition_once)));
  });

  group.finish();
}

criterion_group!(benches, bench_count_solutions);
criterion_main!(benches);
