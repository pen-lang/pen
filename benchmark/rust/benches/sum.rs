use criterion::{black_box, criterion_group, criterion_main, Bencher, Criterion};

const ITERATION_COUNT: usize = 100_000_000;

fn sum(bencher: &mut Bencher) {
    bencher.iter(|| {
        let mut x = ITERATION_COUNT as f64;
        let mut sum = 0.0;

        while x != 0.0 {
            sum = black_box(sum + x);
            x -= 1.0;
        }

        let _ = sum;
    });
}

fn benchmark(criterion: &mut Criterion) {
    criterion.bench_function("sum", sum);
}

criterion_group!(benchmark_group, benchmark);
criterion_main!(benchmark_group);
